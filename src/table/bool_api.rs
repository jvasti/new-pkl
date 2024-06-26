use crate::{PklResult, PklValue};
use std::ops::Range;

// macro_rules! generate_method {
//     ($name:expr;$args:expr; $($arg_index:literal : $arg_type:ident),+; $action:block $range:expr) => {{
//         let name: &str = $name;
//         let number_of_args: usize = count_args!($($arg_index),+);
//         let args: &Vec<PklValue<'_>> = $args;

//         if $args.len() != number_of_args {
//             return Err((
//                 format!(
//                     "Boolean expects '{}' method to take exactly {} argument(s)",
//                     name, number_of_args
//                 ),
//                 $range,
//             ));
//         }

//         $(
//             if args[$arg_index].get_type() != stringify!($arg_type) {
//                 return Err((
//                     format!(
//                         "{} method expects argument at index {} to be of type {}, but found {}",
//                         name, $arg_index, stringify!($arg_type), args[$arg_index].get_type()
//                     ),
//                     $range,
//                 ));
//             }
//         )+

//         $action
//     }};
// }

#[macro_export]
macro_rules! generate_method {
    ($name:expr,$args:expr; $($arg_index:tt : $arg_type:ident),+; $action:expr; $range:expr) => {{
        let name: &str = $name;
        let number_of_args: usize = count_args!($($arg_index),+);
        let args: &Vec<PklValue<'_>> = $args;

        if $args.len() != number_of_args {
            return Err((
                format!(
                    "Boolean expects '{}' method to take exactly {} argument(s)",
                    name, number_of_args
                ),
                $range,
            ));
        }

        $(
            if args[$arg_index].get_type() != stringify!($arg_type) {
                return Err((
                    format!(
                        "{} method expects argument at index {} to be of type {}, but found {}",
                        name, $arg_index, stringify!($arg_type), args[$arg_index].get_type()
                    ),
                    $range,
                ));
            }
        )+

        let args_tuple = (
            $(
                if let PklValue::$arg_type(value) = &args[$arg_index] {
                    *value
                } else {
                    return Err((
                        format!(
                            "{} method expects argument at index {} to be of type {}, but found {}",
                            name, $arg_index, stringify!($arg_type), args[$arg_index].get_type()
                        ),
                        $range,
                    ));
                }
            ),+
        );

        $action(args_tuple)
    }};
}

// Helper macro to count arguments
macro_rules! count_args {
    ($($arg_index:tt),*) => {
        <[()]>::len(&[$(count_args!(@single $arg_index)),*])
    };
    (@single $arg_index:tt) => { () };
}

/// Based on v0.26.0
pub fn match_bool_methods_api<'a, 'b>(
    bool_value: bool,
    fn_name: &'a str,
    args: Vec<PklValue<'b>>,
    range: Range<usize>,
) -> PklResult<PklValue<'b>> {
    match fn_name {
        "xor" => {
            // if args.len() != 1 {
            //     return Err((
            //         format!("Boolean expects 'xor' method to take exactly 1 argument"),
            //         range,
            //     ));
            // }

            // if let Some(other_bool) = args[0].as_bool() {
            //     return Ok((bool_value ^ other_bool).into());
            // } else {
            //     return Err((
            //         format!("1st argument of method 'xor' is expected to be a boolean, argument is of type: `{}`", args[0].get_type()),
            //         range,
            //     ));
            // };

            generate_method!(
                "xor", &args;
                0: Bool;
                |other_bool: bool| {
                        Ok((bool_value ^ other_bool).into())
                };
                range
            )
        }
        "implies" => {
            generate_method!(
                "implies", &args;
                0: Bool;
                |other_bool: bool| {
                        Ok((!bool_value || other_bool).into())
                };
                range
            )
        }
        _ => {
            return Err((
                format!("Boolean does not possess {} method", fn_name),
                range,
            ))
        }
    }
}
