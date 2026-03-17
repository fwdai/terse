pub mod assistant;
pub mod fillers;
pub mod markdown;
pub mod structural;
pub mod substitutions;
pub mod user;
pub mod whitespace;

use crate::tiers::rules::{
    assistant::remove_assistant_boilerplate,
    fillers::remove_fillers,
    markdown::strip_markdown_formatting,
    structural::remove_structural_meta_commentary,
    substitutions::apply_substitutions,
    user::remove_user_boilerplate,
    whitespace::normalize_whitespace,
};

pub fn apply_rules(text: &str, role: &str) -> String {
    let mut result = text.to_string();

    match role {
        "assistant" => {
            result = remove_assistant_boilerplate(&result);
            result = remove_structural_meta_commentary(&result);
        }
        "user" => {
            result = remove_user_boilerplate(&result);
        }
        _ => {}
    }

    result = apply_substitutions(&result);
    result = remove_fillers(&result);
    result = strip_markdown_formatting(&result);
    result = normalize_whitespace(&result);

    result
}
