#[macro_export]
macro_rules! separate_by {
    ($f:expr) => {
        let mut needs_separator = false;

        macro_rules! separate {
            ($action:expr) => {
                if needs_separator {
                    $f;
                }

                $action;

                needs_separator = true;
            };
        }
    };
}
