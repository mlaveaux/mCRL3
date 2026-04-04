use merc_utilities::MercError;

slint::include_modules!();

/// Shows a standard error dialog with the given title and message, blocking
/// input until closed or "ok" is pressed.
pub fn show_error_dialog(title: &str, message: &str) -> Result<(), MercError> {
    let dialog = ErrorDialog::new()?;
    dialog.set_error_title(title.into());
    dialog.set_error_text(message.into());

    // Hide the dialog when the Ok button was pressed.
    {
        let weak_dialog = dialog.as_weak();
        dialog.on_ok_clicked(move || {
            if let Some(dialog) = &weak_dialog.upgrade() {
                dialog.hide().expect("Showing the ErrorDialog failed");
            }
        })
    }

    dialog.show()?;
    Ok(())
}
