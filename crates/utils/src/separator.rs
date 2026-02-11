#[macro_export]
macro_rules! separator {
    ($separate:expr) => {
        let mut needs_separator = false;

        macro_rules! separate {
            ($action:expr) => {
                if needs_separator {
                    $separate;
                }

                $action;

                needs_separator = true;
            };
        }
    };
}
