use mcrl3_aterm::ATerm;
use mcrl3_aterm::Symbol;
use mcrl3_aterm::Term;

fn main() {
    let term = {
        let t = ATerm::constant(&Symbol::new("a", 0));
        t.arg(0)
    };
}