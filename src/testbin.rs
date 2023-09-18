fn main() {
    let mything: isize = 3;
    println!("start at {}", &mything);
}

// use litrs::Literal;
//
// fn main() {
//     // let thing = Literal::parse("b\"abc\"");
//     let thing = Literal::parse("-5.3");
//
//     if let Ok(Literal::String(sl)) = thing {
//         let result: String = String::from(sl.into_value());
//         println!("I got: '{result}'");
//     } else {
//         println!("got nothin' {thing:#?}")
//     }
//
//     let _mybytestr = b"\xaa\xbb 1324";
//
//     // println!("{thing:#?}");
// }

/*************************************\
|* Experiment with memory management *|
\*************************************/
// struct MyStruct(i64);
//
// struct BorrowVec<'a> {
//     objects: Vec<&'a MyStruct>,
// }
// impl<'a> BorrowVec<'a> {
//     fn push(&mut self, obj: Rc<MyStruct>) {
//         let new_rc = obj.clone();
//         self.objects
//             .push(new_rc.clone().as_ref());
//     }
// }
//
// struct RcVec {
//     objects: Vec<Rc<MyStruct>>,
// }
// impl RcVec {
//     fn push(&mut self, obj: Rc<MyStruct>) {
//         self.objects.push(obj.clone())
//     }
// }

// struct Node {
//     pub parent: Option<Rc<Node>>,
//     selfref: Option<Rc<Node>>,
// }
//
// impl Node {
//     fn new(parent: Option<Rc<Node>>) -> Self {
//         let mut node = Node {
//             parent,
//             selfref: None,
//         };
//         node.selfref = Some(Rc::new(node));
//         node
//     }
// }
