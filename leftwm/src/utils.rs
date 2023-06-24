pub mod file_handler;
pub mod log;

pub const fn get_help_template() -> &'static str {
    "\
{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}
"
}
