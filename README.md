# Installation

`cargo install tiny_chatgpt_cli`

# Usage examples

## Regular usage

    ~/i/tiny_chatgpt_cli $ chatgpt
    (i) Enter an empty line to stop
    >>> hello
    Hello! How can I assist you today?
    >>> evaluate 34 + 67
    34 + 67 = 101
    >>>
    Used tokens amount: 59

## Command mode

    ~/i/tiny_chatgpt_cli $ chatgpt cmd
    Input the description of a command: get amount of lines in main.rs

    > wc -l main.rs

    To apply the command, input nothing. To not apply it, input something.

    wc -l main.rs
    136 main.rs
