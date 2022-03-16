#[macro_export]
macro_rules! loop_program {
    ($($e:tt)*) => {{
        format!(
            r#"
var i = 0;
while (i < {num_iter}) {{
  i = i + 1;
}}
"#,
$(
    $e
)*
        )
    }};
}

#[macro_export]
macro_rules! equality_program {
    ($($e:tt)*) => {{
        format!(
            r#"
var i = 0;
while (i < {num_iter}){{
  i = i + 1;
  1 == 1; 1 == 2; 1 == nil; 1 == "str"; 1 == true;
  nil == nil; nil == 1; nil == "str"; nil == true;
  true == true; true == 1; true == false; true == "str"; true == nil;
  "str" == "str"; "str" == "stru"; "str" == 1; "str" == nil; "str" == true;
}}
"#,
$(
    $e
)*
        )
    }};
}

#[macro_export]
macro_rules! instantiation_program{
  ($($e:tt)*) => {{
        format!(
r#"
class Foo {{
  init() {{}}
}}

var i = 0;
while (i < {num_iter}) {{
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  i = i + 1;
}}
"#,
$(
    $e
)*
        )
    }};

}
