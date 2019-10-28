# Tomatina

Turn a [USBButton](http://www.usbbutton.com/) into a
[Pomodoro](https://en.wikipedia.org/wiki/Pomodoro_Technique) timer.

## Installation

### macOS

Tomatina will configure the USBButton to output a keypress of "ctrl+opt+cmd+t" for short button presses
and "ctrl+opt+cmd+u" for long button presses. To be able to observe these signals, we create
keyboard shortcuts that trigger Automator services that write to a named pipe. To install these
Automator services and set up the keyboard shortcuts, run the following:

```
$ open config/macOS/workflows/tomatina-primary.workflow
$ open config/macOS/workflows/tomatina-secondary.workflow
$ defaults write pbs NSServicesStatus -dict-add '"(null) - tomatina-primary - runWorkflowAsService"' '{key_equivalent = "^~@t";}'
$ defaults write pbs NSServicesStatus -dict-add '"(null) - tomatina-secondary - runWorkflowAsService"' '{key_equivalent = "^~@u";}'
```

Then install and run Tomatina with:

```
$ cargo build --release
$ install target/release/tomatina /usr/local/bin/
$ tomatina
```
