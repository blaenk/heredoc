#![feature(phase)]

#[phase(plugin)]
extern crate heredoc;

#[test]
fn heredoc() {
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

  assert_eq!(post,
"---
something = lol
another = hi
---

this is a test for something

this is a codeblock:

    some code
    other code

as you can see the code is sloppy");
}

#[test]
fn join() {
  let re = join!(
    heredoc! {r"
      (?ms)
      \A---\s*\n
      (?P<metadata>.*?\n?)
      ^---\s*$
      \n?
      (?P<body>.*)
    "});

  assert_eq!(re, r"(?ms)\A---\s*\n(?P<metadata>.*?\n?)^---\s*$\n?(?P<body>.*)");
}

