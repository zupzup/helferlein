# Helferlein

It helps.

## What is it?

* It's a GUI tool for doing accounting and invoicing focused on simplicity and one-person companies

## What can it do?

* It can do basic accounting and invoicing for Austria in Euro (but can be extended for other countries as well)

## How can I use it?

* Build it locally using `cargo build --release` and move the `./target/release/helferlein` binary to wherever suits you
* Upon first starting the tool, it will prompt you for a `data folder`, where all data will be stored.
    * It will also create a `config.toml` configuration file in your config dir
        * Linux: /home/alice/.config
        * Windows: C:\Users\Alice\AppData\Roaming
        * MacOs: /Users/Alice/Library/Application Support
* Then the app shows you a welcome screen and you can use the `accounting`, `invoice` and `settings` menu points
    * Accounting - here you can do your accounting and export it to PDF
        * All accounting data is saved, together with the files, in your data directory in a file based database
        * If you export to PDF, the files of the exported items are put into a folder besides the PDF
    * Invoice - here you can create invoices and export them to PDF. You can also save invoice as re-usable templates.
    * Settings - here you can set the data directory, program to open files and the language of the app

## Limitations

* The PDF rendering code is very much suited to a specific layout and has strong limitations in terms of flexibility
* The tool is optimized for Austrian accounting and invoicing
* Helferlein only supports German and English as languages

## Contribution

* If you find a bug, I'll be grateful if you report it
* I use this tool for my own accounting and invoicing needs and only published it for other people to re-use and learn from
    * The goal is not to build a general tool that is suitable for everyone
    * If you need additional features, I'd encourage you to fork and implement them in the way need them

