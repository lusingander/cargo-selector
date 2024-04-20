#[macro_export]
macro_rules! key_code {
    ( $code:path ) => {
        crossterm::event::KeyEvent { code: $code, .. }
    };
}

#[macro_export]
macro_rules! key_code_char {
    ( $c:ident ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            ..
        }
    };
    ( $c:expr, Ctrl ) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($c),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            ..
        }
    };
}
