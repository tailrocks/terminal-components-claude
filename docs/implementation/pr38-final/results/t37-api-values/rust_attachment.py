import re
from dataclasses import dataclass


@dataclass(frozen=True)
class Token:
    value: str
    line: int


@dataclass(frozen=True)
class Scope:
    kind: str
    name: str


def module_name(path):
    match = re.match(r'crates/([^/]+)/src/(.*)\.rs$', path)
    if not match:
        return path
    crate = match.group(1).replace('-', '_')
    parts = match.group(2).replace('\\', '/').split('/')
    if parts[-1] in ('lib', 'main', 'mod'):
        parts.pop()
    if not parts:
        return crate
    return '::'.join([crate, *parts])


def _advance_line(text, line):
    return line + text.count('\n')


def tokenize(source):
    result = []
    index = 0
    line = 1
    size = len(source)
    while index < size:
        char = source[index]
        if char.isspace():
            if char == '\n':
                line += 1
            index += 1
            continue
        if source.startswith('//', index):
            end = source.find('\n', index)
            if end < 0:
                break
            index = end
            continue
        if source.startswith('/*', index):
            start = index
            index += 2
            depth = 1
            while index < size and depth:
                if source.startswith('/*', index):
                    depth += 1
                    index += 2
                elif source.startswith('*/', index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            line = _advance_line(source[start:index], line)
            continue
        raw = re.match(r'(?:br|r)(#+)?"', source[index:])
        if raw:
            hashes = raw.group(1) or ''
            start = index
            index += raw.end()
            close = '"' + hashes
            end = source.find(close, index)
            index = size if end < 0 else end + len(close)
            result.append(Token('<literal>', line))
            line = _advance_line(source[start:index], line)
            continue
        if char in 'bc' and index + 1 < size and source[index + 1] == '"':
            start = index
            index += 2
            escaped = False
            while index < size:
                current = source[index]
                index += 1
                if current == '\n':
                    line += 1
                if escaped:
                    escaped = False
                elif current == '\\':
                    escaped = True
                elif current == '"':
                    break
            result.append(Token('<literal>', line - source[start:index].count('\n')))
            continue
        if char == '"':
            start = index
            index += 1
            escaped = False
            while index < size:
                current = source[index]
                index += 1
                if current == '\n':
                    line += 1
                if escaped:
                    escaped = False
                elif current == '\\':
                    escaped = True
                elif current == '"':
                    break
            result.append(Token('<literal>', line - source[start:index].count('\n')))
            continue
        if char == "'":
            start = index
            cursor = index + 1
            if cursor < size and source[cursor] == '\\':
                cursor += 1
                while cursor < size:
                    if source[cursor] == '\\':
                        cursor += 2
                    elif source[cursor] == "'":
                        index = cursor + 1
                        break
                    else:
                        cursor += 1
            elif cursor + 1 < size and source[cursor + 1] == "'":
                index = cursor + 2
            if index != start:
                result.append(Token('<literal>', line))
                continue
            result.append(Token("'", line))
            index += 1
            continue
        identifier = re.match(r'[A-Za-z_][A-Za-z0-9_]*', source[index:])
        if identifier:
            value = identifier.group(0)
            result.append(Token(value, line))
            index += len(value)
            continue
        number = re.match(r'[0-9][A-Za-z0-9_]*', source[index:])
        if number:
            value = number.group(0)
            result.append(Token(value, line))
            index += len(value)
            continue
        pair = next((pair for pair in ('::', '->', '=>', '..', '&&', '||', '==', '!=', '<=', '>=') if source.startswith(pair, index)), None)
        if pair:
            result.append(Token(pair, line))
            index += len(pair)
        else:
            result.append(Token(char, line))
            index += 1
    return result


def _normal_path(tokens):
    result = []
    depth = 0
    for token in tokens:
        if token == '<':
            depth += 1
            continue
        if token == '>':
            depth = max(0, depth - 1)
            continue
        if depth:
            continue
        if token == 'where':
            break
        if token in ('&', '*', 'mut', 'const', 'dyn', 'impl', "'"):
            continue
        if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', token) or token == '::':
            result.append(token)
    return ''.join(result).strip(':')


def _impl_scope(header):
    impl_index = max(i for i, token in enumerate(header) if token == 'impl')
    tail = header[impl_index + 1:]
    if tail and tail[0] == '<':
        depth = 0
        end = 0
        for end, token in enumerate(tail):
            if token == '<':
                depth += 1
            elif token == '>':
                depth -= 1
                if depth == 0:
                    end += 1
                    break
        tail = tail[end:]
    depth = 0
    split = None
    stop = len(tail)
    for index, token in enumerate(tail):
        if token == '<':
            depth += 1
        elif token == '>':
            depth = max(0, depth - 1)
        elif depth == 0 and token == 'where':
            stop = index
            break
        elif depth == 0 and token == 'for' and split is None:
            split = index
    tail = tail[:stop]
    if split is not None and split < stop:
        trait = _normal_path(tail[:split])
        target = _normal_path(tail[split + 1:])
        return f'{trait} for {target}'
    return _normal_path(tail)


def _classify(header):
    for index in range(len(header) - 1, -1, -1):
        value = header[index]
        if value == 'impl':
            return Scope('impl', _impl_scope(header))
        if value in ('mod', 'trait', 'enum', 'struct', 'union'):
            name = next((item for item in header[index + 1:] if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', item)), '')
            return Scope(value, name)
    return None


class RustSource:
    def __init__(self, path, source):
        self.path = path
        self.source = source
        self.lines = source.splitlines()
        self.tokens = tokenize(source)
        self.before_token = []
        stack = []
        header = []
        for token in self.tokens:
            self.before_token.append(tuple(scope for scope in stack if scope is not None))
            if token.value == '{':
                stack.append(_classify(header))
                header = []
            elif token.value == '}':
                if stack:
                    stack.pop()
                header = []
            elif token.value == ';':
                header = []
            else:
                header.append(token.value)

    def _first_token(self, line):
        return next((index for index, token in enumerate(self.tokens) if token.line == line), None)

    def scopes_at_line(self, line):
        token_index = self._first_token(line)
        if token_index is not None:
            return list(self.before_token[token_index])
        prior = [index for index, token in enumerate(self.tokens) if token.line < line]
        return list(self.before_token[prior[-1]]) if prior else []

    def next_item_line(self, after_line):
        index = next((i for i, token in enumerate(self.tokens) if token.line > after_line), len(self.tokens))
        while index < len(self.tokens) and self.tokens[index].value == '#':
            cursor = index + 1
            if cursor < len(self.tokens) and self.tokens[cursor].value == '!':
                cursor += 1
            if cursor >= len(self.tokens) or self.tokens[cursor].value != '[':
                break
            depth = 0
            while cursor < len(self.tokens):
                if self.tokens[cursor].value == '[':
                    depth += 1
                elif self.tokens[cursor].value == ']':
                    depth -= 1
                    if depth == 0:
                        cursor += 1
                        break
                cursor += 1
            index = cursor
        return index if index < len(self.tokens) else None

    def _declaration_name(self, token_index, scopes):
        line = self.tokens[token_index].line
        values = [token.value for token in self.tokens[token_index:] if token.line == line]
        cursor = 0
        while cursor < len(values) and values[cursor] == 'pub':
            cursor += 1
            if cursor < len(values) and values[cursor] == '(':
                depth = 0
                while cursor < len(values):
                    if values[cursor] == '(':
                        depth += 1
                    elif values[cursor] == ')':
                        depth -= 1
                        if depth == 0:
                            cursor += 1
                            break
                    cursor += 1
        while cursor < len(values) and values[cursor] in ('async', 'unsafe', 'default', 'extern', 'crate'):
            modifier = values[cursor]
            cursor += 1
            if modifier == 'extern' and cursor < len(values) and values[cursor] == '<literal>':
                cursor += 1
        if cursor + 1 < len(values) and values[cursor] == 'const' and values[cursor + 1] == 'fn':
            cursor += 1
        if cursor < len(values) and values[cursor] in ('fn', 'struct', 'enum', 'trait', 'type', 'mod', 'const', 'static', 'union'):
            kind = values[cursor]
            name = next((value for value in values[cursor + 1:] if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', value)), None)
            if name:
                return kind, name
        if scopes and scopes[-1].kind in ('enum', 'struct'):
            first = values[cursor] if cursor < len(values) else ''
            second = values[cursor + 1] if cursor + 1 < len(values) else ''
            if re.fullmatch(r'[A-Z][A-Za-z0-9_]*', first) and second in (',', '(', '{', '='):
                return 'variant', first
            if cursor + 1 < len(values) and re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', values[cursor]) and values[cursor + 1] == ':':
                return 'field', values[cursor]
        return None, None

    def item_at_token(self, token_index):
        token = self.tokens[token_index]
        scopes = list(self.before_token[token_index])
        kind, symbol = self._declaration_name(token_index, scopes)
        if not symbol:
            return None
        owner_modules = [scope for scope in scopes if scope.kind == 'mod']
        owner_types = [scope for scope in scopes if scope.kind in ('impl', 'trait', 'enum', 'struct')]
        chain = [f'{scope.kind}:{scope.name}' for scope in owner_modules + owner_types]
        parts = module_name(self.path).split('::')
        parts.extend(scope.name for scope in owner_modules)
        parts.extend(scope.name for scope in owner_types)
        parts.append(symbol)
        line = token.line
        signature = self.lines[line - 1].strip()
        return {
            'kind': 'rust_item',
            'name': '::'.join(parts),
            'signature': signature,
            'source_line': line,
            'scope_chain': chain,
            'item_kind': kind,
        }

    def attached_item(self, text, end_line):
        if any(line.lstrip().startswith('//!') for line in text):
            line_number = next((index for index, line in enumerate(self.lines, 1) if line.lstrip().startswith('//!')), 1)
            modules = [scope for scope in self.scopes_at_line(line_number) if scope.kind == 'mod']
            name = '::'.join([module_name(self.path), *(scope.name for scope in modules)])
            anchor = None
            for line in self.lines[end_line:]:
                stripped = line.strip()
                if re.match(r'pub\s+(?:mod|trait|struct|enum|type|const|fn)\b', stripped):
                    anchor = stripped
                    break
            return {
                'kind': 'module_documentation',
                'name': name,
                'signature': 'module-level Rustdoc',
                'anchor_declaration': anchor,
                'source_line': line_number,
                'scope_chain': [f'module:{scope.name}' for scope in modules],
            }
        token_index = self.next_item_line(end_line)
        if token_index is None:
            return {'kind': 'rust_item', 'name': f'{module_name(self.path)}::<unresolved>', 'signature': '', 'source_line': None, 'scope_chain': [], 'item_kind': None}
        item = self.item_at_token(token_index)
        if item is None:
            line = self.tokens[token_index].line
            return {'kind': 'rust_item', 'name': f'{module_name(self.path)}::<unresolved at {line}>', 'signature': self.lines[line - 1].strip(), 'source_line': line, 'scope_chain': [], 'item_kind': None}
        return item

    def find_item(self, name, signature):
        matches = []
        seen_lines = set()
        for index, token in enumerate(self.tokens):
            if token.line in seen_lines:
                continue
            seen_lines.add(token.line)
            if self.lines[token.line - 1].strip() != signature:
                continue
            item = self.item_at_token(index)
            if item and item['name'] == name:
                matches.append(item)
        return matches
