#![feature(phase)]

#[phase(plugin)]
extern crate heredoc;

#[test]
fn testit() {
  // let hd = join!(
  //   heredoc! {r"
  //     (?ms)
  //     \A---\s*\n
  //     (?P<metadata>.*?\n?)
  //     ^---\s*$
  //     \n?
  //     (?P<body>.*)
  //   "});
  // println!(">>{}<<", hd);

  let post = heredoc! {"
    ---
    something = lol
    another = hi
    ---

    this is a test for something

    this is a codeblock:

        some code
        other code

    as you can see the code is sloppy
  "};
  println!(">>{}<<", post);

  assert!(true);
}

