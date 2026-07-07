# Opening a PDF

## File picker

1. Launch the app.
2. Click **Open PDF…** on the dashboard.
3. Select a `.pdf` file from the file picker.

## Drag and drop

Drop a single `.pdf` file onto the dashboard window. The app validates
the file header before loading — non-PDF files with a `.pdf` extension
are rejected with a clear error message.

> Only one file can be opened at a time. Dropping multiple files shows
> an error and no file is loaded.

## From the session history

Documents opened during the current session are listed on the dashboard.
Click any entry to reopen it. The history is cleared when the app closes.

## Password-protected PDFs

If a PDF requires a password, the app shows a password prompt. Enter the
password and choose **Open**. If the password is not accepted, the prompt stays
open and the password field is cleared.

Choose **Cancel** or close the prompt to keep the current dashboard or viewer
unchanged. Passwords are not saved.

## Error handling

| Situation | Message shown |
|-----------|--------------|
| File not found | "The file could not be found." |
| Not a file (directory) | "That item is not a file." |
| Wrong extension | "Only .pdf files can be opened." |
| Not a valid PDF | "This file is not a valid PDF document." |
| Password not accepted | "The password was not accepted." |
