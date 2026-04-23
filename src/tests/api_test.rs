use crate::parse_latex;

#[test]
fn test_unified_api() {
    let math = "a & b \\\\ c & d";
    let ast = parse_latex(math).unwrap();
    println!("{:#?}", ast);
}
