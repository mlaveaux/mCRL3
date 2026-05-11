use merc_aterm::ATermInt;

fn main() {
    let term = {
        let i = ATermInt::new(42);
        i.inner()
    };

    // Have some side effect    
    println!("Term: {:?}", term);
}