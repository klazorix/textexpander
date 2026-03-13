# TextExpander
A lightweight peice of software developed by @klazorix that allows you to type commands (triggers) and have them expanded into set text of your choice.

## Get the latest version
To get the latest, feature-rich, and stable version of TextExpander, visit [this link](https://github.com/klazorix/textexpander/releases/latest) and download the `Source Code (zip)` file.  
For further instructions, refer to the documentation.

## Which version is right for you?
| Version | Editing Expansions             | Managing Expansions                                | Viewing Expansions                  | Automatic Updates | CPU Usage    | RAM Usage    | Storage Required | Status            | Notes                                        |
| ------- | ------------------------------ | -------------------------------------------------- | ----------------------------------- | ----------------- | ------------ | ------------ | ---------------- | ----------------- | -------------------------------------------- |
| v4.0.0  | UI-based snippet editor        | Modular system for snippets, triggers, and hotkeys | Dedicated snippet manager interface | ✅                 | Unknown      | Unknown      | Unknown          | 🚧 In Development | No production release available yet          |
| v3.0.0  | Basic UI editor (v2 interface) | Same expansion management system as v2             | Basic UI viewer (v2 interface)      | ✅ (Legacy)        | Not measured | Not measured | Not measured     | ❌ Cancelled         | Development cancelled after Beta Release 1 in favour of v4.0.0.   |
| v2.0.0  | Built-in UI editor             | Integrated expansion manager                       | Built-in UI viewer                  | ✅ (Legacy)        | <0.5%        | ~43 MB       | ~50 MB           | 🧱 Legacy         | Current stable release but no longer updated |
| v1.0.0  | JSON file editing              | Manual file-based management                       | Console output                      | ❌                 | <1%          | ~13 MB       | ~7 MB            | ❌ Unsupported     | Initial release                              |

_Supported refers to wether the version still receives bug and security fixes_  
_JSON file editing requires you to install an IDE and have a fundamental understanding of JSON as a lanaguage._


## Security Warnings on Installation
TextExpander is packaged using **PyInstaller OneDirectory** (which is the source of a majority of the files inside the `_internal` folder).
Due to this, as well as the fact that the main application is a .exe file, most Antivirus softwares, including Windows Defender, will block you from downloading and/or running the file.
These warnings are nothing to worry about, as the only reason for the alert is because the code is unsigned (not approved by microsoft).

## Documentation & Support
See the documentation here to learn more:
https://docs.klazorix.com/text-expander/
