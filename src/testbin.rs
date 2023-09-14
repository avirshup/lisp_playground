use litrs::Literal;

fn main() {
    // let thing = Literal::parse("b\"abc\"");
    let thing = Literal::parse("-5.3");

    if let Ok(Literal::String(sl)) = thing {
        let result: String = String::from(sl.into_value());
        println!("I got: '{result}'");
    } else {
        println!("got nothin' {thing:#?}")
    }

    let _mybytestr = b"\xaa\xbb 1324";

    // println!("{thing:#?}");
}
