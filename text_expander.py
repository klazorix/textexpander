global typed_buffer
import subprocess
import sys
import time
import os
import json
import datetime
import pyperclip
import requests

try:
    import keyboard
    import pyautogui
except ImportError:
    print("\033[33mInstalling dependencies, please wait...\033[0m")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "keyboard"])
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyautogui"])
    import keyboard
    import pyautogui

DATA_FILE = "text_expander_data.json"
VERSION = "v1.2.3"


def clear():
    """Clear console screen."""
    os.system("cls" if os.name == "nt" else "clear")


def check_updates():
    try:
        latest = requests.get("https://api.github.com/repos/klazorix/textexpander/releases/latest").json().get("tag_name", "")
        if VERSION != latest:
            print("--------------------------------------------------------------------\n")
            print(f"A new version of TextExpander is available: \033[31m{VERSION}\033[0m -> \033[32m{latest}\033[0m\n")
    except requests.exceptions.RequestException as e:
        print("--------------------------------------------------------------------\n")
        print(f"\033[31mTextExpander encountered an error when trying to check for updates.\033[0m\n")


def load_data():
    """Load data from JSON file, or raise an error if the file is missing."""
    if not os.path.exists(DATA_FILE):
        clear()
        print(f"\033[31mAn error occurred when trying to load your data file. \nEnsure the executable is in the same directory as a correctly formatted .json file called 'text_expander_data'.\033[0m")
        print("\nRefer to the docs for help formatting your JSON file:\n\033[90mhttps://docs.klazorix.com/text-expander/\033[0m\n")
        sys.exit(1)
    
    with open(DATA_FILE, "r") as file:
        return json.load(file)


data = load_data()
variables = data.get("variables", {})
text_expansions = data.get("text_expansions", {})


def replace_placeholders(text):
    """Replace placeholders in text with corresponding values from variables."""
    for var_name, var_value in variables.items():
        placeholder = f"{{{var_name}}}"
        text = text.replace(placeholder, var_value)

    text = text.replace("{date}", datetime.datetime.now().strftime("%d/%m/%Y"))
    text = text.replace("{time}", datetime.datetime.now().strftime("%H:%M"))
    text = text.replace("{clipboard}", pyperclip.paste())
    return text


def print_expansion_list():
    """Print available text expansions."""
    clear()

    print("""
  _______        _   ______                            _           
 |__   __|      | | |  ____|                          | |          
    | | _____  _| |_| |__  __  ___ __   __ _ _ __   __| | ___ _ __ 
    | |/ _ \ \/ / __|  __| \ \/ / '_ \ / _` | '_ \ / _` |/ _ \ '__|
    | |  __/>  <| |_| |____ >  <| |_) | (_| | | | | (_| |  __/ |   
    |_|\___/_/\_\\__|______/_/\_\ .__/ \__,_|_| |_|\__,_|\___|_|   
                                | |                                
                                |_|                                              
    """)
    check_updates()
    print("--------------------------------------------------------------------\n")
    for trigger, expansion in text_expansions.items():
        print(f"{trigger: <15}: {expansion}")
    print("\n--------------------------------------------------------------------\n")
    print("Documentation: \033[90mhttps://docs.klazorix.com/text-expander/\033[0m")
    print(f"JSON Data File Location: \033[90m{os.path.abspath(DATA_FILE)}\033[0m")
    print("\n--------------------------------------------------------------------\n")


print_expansion_list()
typed_buffer = ""


def handle_key_event(event):
    """Handle keyboard input events for text expansion."""
    global typed_buffer

    try:
        if event.event_type == "down":
            if len(event.name) == 1 and event.name != "/":
                typed_buffer += event.name
            elif event.name in ["space", "enter"]:
                if typed_buffer in text_expansions:
                    expansion = replace_placeholders(text_expansions[typed_buffer])

                    pyautogui.hotkey('ctrl', 'a')
                    pyautogui.press('delete')

                    keyboard.write(expansion)
                    time.sleep(0.1)

                typed_buffer = ""

            elif event.name == "backspace" and typed_buffer:
                typed_buffer = typed_buffer[:-1]

            else:
                typed_buffer = ""

    except Exception as e:
        print(f"Error occurred: {e}")


keyboard.hook(handle_key_event)
print("\033[32mText Expander is running.\033[0m Type the trigger and \033[38;5;214mpress space to expand\033[0m. Press Ctrl+C to exit.")

try:
    keyboard.wait()
except KeyboardInterrupt:
    print("\nText Expander stopped.")