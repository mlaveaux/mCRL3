use aterm::binary_io::{write_term_to_binary_stream, read_term_from_binary_stream, Aterm, FunctionSymbol};
use std::io::Cursor;

#[test]
fn test_binary_io() {
    let term = Aterm::Appl(
        FunctionSymbol {
            name: "f".to_string(),
            arity: 2,
        },
        vec![Aterm::Int(1), Aterm::Int(2)],
    );

    let mut buffer = Vec::new();
    write_term_to_binary_stream(&term, &mut buffer);

    let cursor = Cursor::new(buffer);
    let read_term = read_term_from_binary_stream(cursor);

    assert_eq!(term, read_term);
}