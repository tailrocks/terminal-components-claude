use termrock::Theme;
use termrock_consumer_example::RecipeBox;

fn main() -> std::io::Result<()> {
    termrock::run(RecipeBox::default(), Theme::junie())
}
