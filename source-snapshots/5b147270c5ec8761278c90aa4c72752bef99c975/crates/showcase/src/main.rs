//! `showcase` binary entry point.

fn main() -> std::io::Result<()> {
    showcase_ui::run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_showcase_binary_entry() {
        // Sanity test verifying showcase binary compiles and links.
    }
}
