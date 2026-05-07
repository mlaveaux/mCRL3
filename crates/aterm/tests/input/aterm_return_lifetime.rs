use merc_aterm::ATermInt;
use merc_aterm::Term;

fn main() {
    let term = {
        let i = ATermInt::new(42);
        (*i).copy();
    };

    // Have some side effect    
    println!("Term: {:?}", term);
}