"""Independent tiny production source used only by the runner qualification suite."""


class Props:
    def __init__(self, disabled=False):
        self.disabled = disabled

    def enabled(self):
        return not self.disabled


class Widget:
    def draw(self, value, palette):
        return {"text": str(value), "foreground": palette, "owner": "Widget"}


class App:
    def __init__(self, value, palette, disabled=False, width=8):
        self.value = value
        self.palette = palette
        self.props = Props(disabled)
        self.width = width
        self.widget = Widget()

    def update(self, key):
        if self.props.enabled() and key == "+":
            self.value += 1

    def draw(self):
        # Custom art is legitimate; product controls still use Widget.draw.
        return {"width": self.width, "custom_art": "*", "control": self.widget.draw(self.value, self.palette)}
