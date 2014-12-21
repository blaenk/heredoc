#![feature(plugin_registrar)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast::{TokenTree, ExprLit, LitStr};
use syntax::ast::StrStyle::{CookedStr, RawStr};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse::token;
use syntax::parse::token::{Token, Lit};
use syntax::parse;

pub fn first_non_whitespace_multiline(slice: &str) -> Option<uint> {
  let mut indent = 0;

  for ch in slice.chars() {
    if !ch.is_whitespace() {
      return Some(indent);
    } else if ch == '\n' || ch == '\r' {
      indent = 0;
    } else {
      indent += 1;
    }
  }

  return None;
}

pub fn format(slice: &str) -> String {
  let adjusted = if slice.char_at(0) == '\n' {
    slice.slice_from(1)
  } else {
    slice
  };

  let pos = first_non_whitespace_multiline(adjusted);
  let mut string = String::new();

  if let Some(pos) = pos {
    let lines = adjusted.lines().map(|line| {
      let len = line.len();
      if len > 0 && line.as_bytes()[len - 1] == b'\r' {
        if len > pos {
          (line.slice(pos, len - 1), "\r\n")
        } else {
          ("", "\r\n")
        }
      } else {
        if len > pos {
          (line.slice_from(pos), "\n")
        } else {
          ("", "\n")
        }
      }
    }).collect::<Vec<(&str, &str)>>();

    let last = lines.len() - 1;

    for (index, &(line, ending)) in lines.iter().enumerate() {
      string.push_str(line);

      // don't emit a newline if:
      //   - it's the last line
      //   - it's the penultimate line and the last line is empty
      if index != last &&
         !(index == (last - 1) && lines[last].0.is_empty()) {
        string.push_str(ending);
     }
    }
  }

  string
}

#[plugin_registrar]
#[doc(hidden)]
pub fn registrar(reg: &mut Registry) {
  reg.register_macro("heredoc", expand_heredoc);
}

// notes:
//
// https://github.com/rust-lang/rust/blob/45cbdec4174778bf915f17561ef971c068a7fcbc/src/libsyntax/ext/concat.rs

fn expand_heredoc(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
                 -> Box<MacResult + 'static> {
  if tts.len() != 1 {
    cx.span_err(sp, "heredoc requires a single string literal");
    return DummyResult::expr(sp);
  }

  // StrRaw(string, num_hashes)
  // TtToken(
  //  Span {
  //    lo: BytePos(114),
  //    hi: BytePos(222),
  //    expn_id: ExpnId(4294967295)
  //  },
  //  Literal(
  //    StrRaw(
  //      "the str"(67), <-- 67 is the len?
  //      0),            <-- num_hashes
  //    None))

  if let TokenTree::TtToken(span, Token::Literal(lit, _name)) = tts[0] {
    match lit {
      Lit::Str_(string) => {
        let formatted = format(string.as_str());
        return MacExpr::new(
          cx.expr_lit(
            span,
            LitStr(
              token::intern_and_get_ident(
                parse::str_lit(formatted.as_slice()).as_slice()),
              CookedStr)));
      },
      Lit::StrRaw(string, hashes) => {
        let formatted = format(string.as_str());
        return MacExpr::new(
          cx.expr_lit(
            span,
            LitStr(
              token::intern_and_get_ident(
                parse::raw_str_lit(formatted.as_slice()).as_slice()),
              RawStr(hashes))));
      },
      _ => {
        cx.span_err(span, "not a string literal");
        return DummyResult::expr(span);
      },
    }
  } else {
    cx.span_err(sp, "incorrect token");
    return DummyResult::expr(sp);
  }

  MacExpr::new(
    cx.expr_str(
      sp,
      token::intern_and_get_ident("testing")))
}

fn expand_join(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
                 -> Box<MacResult + 'static> {
  // let mut parser =
  //   parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), tts.to_vec());
  let mut parser = cx.new_parser_from_tts(tts);
  let heredoc_expr = cx.expander().fold_expr(parser.parse_expr());
  match heredoc_expr.node {
    ExprLit(ref lit) => {
      match lit.node {
        // LitStr(InternedString, StrStyle)
        // StrStyle:
        //   CookedStr is "lit"
        //   RawStr(num_hashes) is r#"lit"#
        LitStr(ref s, _) => {
          let slice = s.get();
          let mut joined = String::new();
          for line in slice.lines_any() {
            joined.push_str(line);
          }

          return MacExpr::new(
            cx.expr_str(
              sp,
              token::intern_and_get_ident(joined.as_slice())))
        },

        // LitStr(_, CookedStr) => {
        //   cx.span_err(heredoc_expr.span, "was given a cooked string");
        //   return DummyResult::expr(sp);
        // }

        // LitBinary(Rc<Vec<u8>>)
        // LitBinary(ref s) => _,
        _ => {
          cx.span_err(heredoc_expr.span, "expected string literal");
          return DummyResult::expr(sp);
        }
      }
    },
    _ =>  {
      cx.span_err(heredoc_expr.span, "expected string literal");
      return DummyResult::expr(sp);
    }
  };
}

