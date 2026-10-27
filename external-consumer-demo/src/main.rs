





use external_consumer_demo::RecipeBox;
use termrock::Theme;

fn main() -> std::io::Result<()> {
    termrock::run(RecipeBox::default(), Theme::junie())
}
