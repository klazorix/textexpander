import os
import sys
import subprocess
import threading
import time
import pyperclip
import requests
import datetime
import sqlite3
import webbrowser

try:
    import tkinter as tk
    from tkinter import ttk, messagebox
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "tkinter"])
    import tkinter as tk
    from tkinter import ttk, messagebox

try:
    import psutil
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "psutil"])
    import psutil

try:
    import zipfile
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "zipfile"])
    import zipfile

try:
    import io
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "io"])
    import io

try:
    import keyboard
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "keyboard"])
    import keyboard

try:
    import pyautogui
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyautogui"])
    import pyautogui

try:
    from pystray import Icon, MenuItem, Menu
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pystray"])
    from pystray import Icon, MenuItem, Menu

try:
    from PIL import Image, ImageDraw
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "Pillow"])
    from PIL import Image, ImageDraw

if hasattr(sys, '_MEIPASS'):
    SCRIPT_DIR = sys._MEIPASS
else:
    SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

DATA_FILE = os.path.join(SCRIPT_DIR, "text_expander_data.db")
TRAY_ICON = Image.open(os.path.join(SCRIPT_DIR, "icon.png"))
VERSION = "v2.0.0"

if (os.path.basename(__file__) != "text_expander.py") and (os.path.basename(__file__) != "text_expander.exe"):
    messagebox.showinfo(
        f"TextExpander {VERSION} - Notifications",
        f"An error occured. The executable is not called text_expander.py",
        icon="error"
    )


for proc in psutil.process_iter(['pid', 'name']):
        try:
            if proc.info['name'] == 'python.exe' and 'text_expander' in proc.info['cmdline'][0]:
                proc.terminate()
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass

def check_updates():
    try:
        latest = requests.get("https://api.github.com/repos/klazorix/textexpander/releases/latest").json().get("tag_name", "")
        if VERSION != latest:
            result = messagebox.askquestion(
                f"TextExpander {VERSION} - Update Available",
                f"TextExpander {latest} is now available!\nWould you like to upgrade?",
                icon="warning",
                type=messagebox.YESNOCANCEL,
            )
            if result == "yes":
                messagebox.showinfo(
                    f"TextExpander {VERSION} - Notifications",
                    f"To ensure you get the latest version of TextExpander, we're going to install the latest version of the TextExpander Updates Script.",
                    icon="info"
                )

                repo_url = 'https://api.github.com/repos/klazorix/textexpander/releases/latest'
                response = requests.get(repo_url)
                data = response.json()

                tag = data['tag_name'].lstrip('v')
                zip_url = f'https://github.com/klazorix/textexpander/archive/refs/tags/{data["tag_name"]}.zip'

                zip_data = requests.get(zip_url)

                script_dir = os.path.dirname(os.path.abspath(__file__))
                exe_path = os.path.join(script_dir, '_internal', 'update_installer.pyw')

                if os.path.exists(exe_path):
                    os.remove(exe_path)

                with zipfile.ZipFile(io.BytesIO(zip_data.content)) as zf:
                    exe_file = f'textexpander-{tag}/_internal/update_installer.pyw'
                    if exe_file in zf.namelist():
                        exe_path = os.path.join(script_dir, '_internal', 'update_installer.pyw')
                        with zf.open(exe_file) as exe:
                            with open(exe_path, 'wb') as f:
                                f.write(exe.read())

                messagebox.showinfo(
                    f"TextExpander {VERSION} - Notifications",
                    f"You are about to run our updater script. If prompted, make sure to select 'Python (Windowed)' else the updater will not be able to run.",
                    icon="info"
                )
                try:
                    updater_path = os.path.join(SCRIPT_DIR, "update_installer.pyw")
                    subprocess.Popen(["pythonw", updater_path], shell=True)
                    sys.exit()
                except Exception as e:
                    messagebox.showerror("Error", f"An error occurred while trying to update: {str(e)}")
                    return
            elif result == "cancel":
                sys.exit()

    except requests.exceptions.RequestException as e:
        messagebox.showerror("Error", f"Error checking for updates: {str(e)}")

def initialize_db():
    with sqlite3.connect(DATA_FILE) as connection:
        cursor = connection.cursor()

        cursor.execute(''' 
        CREATE TABLE IF NOT EXISTS variables ( 
            name TEXT PRIMARY KEY, 
            value TEXT 
        )''')

        cursor.execute(''' 
        CREATE TABLE IF NOT EXISTS text_expansions ( 
            trigger TEXT PRIMARY KEY, 
            expansion TEXT 
        )''')

        connection.commit()

def load_data():
    if not os.path.exists(DATA_FILE):
        sys.exit(1)

    with sqlite3.connect(DATA_FILE) as connection:
        cursor = connection.cursor()

        cursor.execute("SELECT name, value FROM variables")
        variables = dict(cursor.fetchall())

        cursor.execute("SELECT trigger, expansion FROM text_expansions")
        text_expansions = dict(cursor.fetchall())

    return variables, text_expansions

variables, text_expansions = load_data()

def replace_placeholders(text):
    for var_name, var_value in variables.items():
        placeholder = f"{{{var_name}}}"
        text = text.replace(placeholder, var_value)

    text = text.replace("{date}", datetime.datetime.now().strftime("%d/%m/%Y"))
    text = text.replace("{time}", datetime.datetime.now().strftime("%H:%M"))
    text = text.replace("{clipboard}", pyperclip.paste())
    return text

def handle_key_event(event):
    global typed_buffer

    try:
        if event.event_type == "down":
            if len(event.name) == 1 and event.name != "/":
                typed_buffer += event.name

            elif event.name in ["space", "enter"]:
                if typed_buffer in text_expansions:
                    expansion = replace_placeholders(text_expansions[typed_buffer])

                    pyautogui.press('backspace', presses=len(typed_buffer) + 1)
                    keyboard.write(expansion)

                typed_buffer = ""

            elif event.name == "backspace" and typed_buffer:
                typed_buffer = typed_buffer[:-1]

            else:
                typed_buffer = ""

    except Exception as e:
        print(f"Error occurred: {e}")

def create_tray_icon():
    tray_icon = Icon("TextExpander", TRAY_ICON, "TextExpander by @klazorix", 
                     Menu(MenuItem("Settings", lambda icon, item: app.restore_from_tray(icon, item)),
                          MenuItem("Exit", lambda icon, item: app.exit_program(icon, item))))
    tray_icon.run()

import tkinter as tk
from tkinter import ttk, messagebox
import sqlite3

def reload_expansions():
    global text_expansions
    with sqlite3.connect(DATA_FILE) as connection:
        cursor = connection.cursor()
        cursor.execute("SELECT trigger, expansion FROM text_expansions")
        text_expansions = dict(cursor.fetchall())

class TextExpanderUI(tk.Tk):
    def __init__(self, tray_icon):
        super().__init__()
        self.tray_icon = tray_icon
        self.withdraw()
        self.title(f"TextExpander {VERSION}")
        self.geometry("600x400")
        self.resizable(False, False)
        self.protocol("WM_DELETE_WINDOW", self.minimize_to_tray)
        self.create_widgets()
        self.load_data()

        self.show_startup_popup()

    def show_startup_popup(self):
        messagebox.showinfo(
            f"TextExpander {VERSION} - Notifications",
            f"TextExpander is now running!\n\nUse your system tray to exit the program or update expansions/variables.",
            icon="info"
        )

    def create_widgets(self):
        self.tab_control = ttk.Notebook(self)
        self.tab_control.pack(fill=tk.BOTH, expand=True)

        self.tab_expansion = ttk.Frame(self.tab_control)
        self.tab_variables = ttk.Frame(self.tab_control)
        self.tab_about = ttk.Frame(self.tab_control)

        self.tab_control.add(self.tab_expansion, text="Text Expansions")
        self.tab_control.add(self.tab_variables, text="Variables")
        self.tab_control.add(self.tab_about, text="About")

        self.text_expansion_tree = ttk.Treeview(
            self.tab_expansion, columns=("Trigger", "Expansion"), show="headings"
        )
        self.text_expansion_tree.heading("Trigger", text="Trigger")
        self.text_expansion_tree.heading("Expansion", text="Expansion")
        self.text_expansion_tree.column("Trigger", width=150, anchor="w")
        self.text_expansion_tree.column("Expansion", width=300, anchor="w")
        self.text_expansion_tree.pack(fill=tk.BOTH, expand=True)

        self.button_frame = ttk.Frame(self.tab_expansion)
        self.button_frame.pack(fill=tk.X, pady=5)

        self.button_inner_frame = ttk.Frame(self.button_frame)
        self.button_inner_frame.pack(pady=3)

        self.edit_button = ttk.Button(self.button_inner_frame, text="Edit", command=self.edit_selected)
        self.edit_button.pack(side=tk.LEFT, padx=5)

        self.delete_button = ttk.Button(self.button_inner_frame, text="Delete", command=self.delete_selected)
        self.delete_button.pack(side=tk.LEFT, padx=5)

        self.add_expansion_button = ttk.Button(self.button_frame, text="Add Expansion", command=self.add_expansion)
        self.add_expansion_button.pack(pady=5)

        self.variables_tree = ttk.Treeview(self.tab_variables, columns=("Name", "Value"), show="headings")
        self.variables_tree.heading("Name", text="Name")
        self.variables_tree.heading("Value", text="Value")
        self.variables_tree.pack(fill=tk.BOTH, expand=True)

        self.tab_variables_buttons_frame = ttk.Frame(self.tab_variables)
        self.tab_variables_buttons_frame.pack(pady=10)

        self.add_variable_button = ttk.Button(self.tab_variables_buttons_frame, text="Add Variable", command=self.add_variable)
        self.add_variable_button.pack(side=tk.LEFT, padx=10)

        # Add Delete button for variables
        self.delete_variable_button = ttk.Button(self.tab_variables_buttons_frame, text="Delete Variable", command=self.delete_selected_variable)
        self.delete_variable_button.pack(side=tk.LEFT, padx=10)

        self.help_text = tk.Label(
            self.tab_about, 
            text=( 
                "Thank you for installing TextExpander.\n"
                "Please report any issues you find via the GitHub.\n\n"
                "If you need help, you can find the docs here:\n"
                "https://docs.klazorix.com/text-expander/getting-started\n\n"
                "Developed by @klazorix\n"
                f"You are running TextExpander {VERSION}"
            ),
            justify="center"
        )
        self.help_text.pack(fill=tk.BOTH, expand=True)

    def load_data(self):
        for row in self.text_expansion_tree.get_children():
            self.text_expansion_tree.delete(row)

        with sqlite3.connect(DATA_FILE) as connection:
            cursor = connection.cursor()

            cursor.execute("SELECT trigger, expansion FROM text_expansions")
            text_expansions = cursor.fetchall()
            for row in text_expansions:
                self.text_expansion_tree.insert("", tk.END, values=row)

        for row in self.variables_tree.get_children():
            self.variables_tree.delete(row)

        with sqlite3.connect(DATA_FILE) as connection:
            cursor = connection.cursor()

            cursor.execute("SELECT name, value FROM variables")
            variables = cursor.fetchall()
            for row in variables:
                self.variables_tree.insert("", tk.END, values=row)

    def edit_selected(self):
        selected_item = self.text_expansion_tree.selection()
        if not selected_item:
            messagebox.showinfo("Error", "Select an item to edit.", icon="warn")
            return
        
        values = self.text_expansion_tree.item(selected_item, "values")
        trigger, expansion = values

        dialog = EditExpansionDialog(self, trigger, expansion)
        self.wait_window(dialog.top)
        self.load_data()
        reload_expansions()

    def delete_selected(self):
        selected_item = self.text_expansion_tree.selection()
        if not selected_item:
            messagebox.showerror("Error", "Please select an item to delete.")
            return

        values = self.text_expansion_tree.item(selected_item, "values")
        trigger = values[0]

        if messagebox.askyesno("Delete", f"Are you sure you want to delete the expansion for '{trigger}'?"):
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("DELETE FROM text_expansions WHERE trigger = ?", (trigger,))
                connection.commit()
            self.load_data()
            reload_expansions()

    def delete_selected_variable(self):
        selected_item = self.variables_tree.selection()
        if not selected_item:
            messagebox.showerror("Error", "Please select a variable to delete.")
            return

        values = self.variables_tree.item(selected_item, "values")
        name = values[0]

        if messagebox.askyesno("Delete", f"Are you sure you want to delete the variable '{name}'?"):
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("DELETE FROM variables WHERE name = ?", (name,))
                connection.commit()
            self.load_data()

    def add_expansion(self):
        dialog = AddExpansionDialog(self)
        self.wait_window(dialog.top)
        self.wait_window(dialog.top)
        self.load_data()

    def add_variable(self):
        dialog = AddVariableDialog(self)
        self.wait_window(dialog.top)
        self.load_data()

    def minimize_to_tray(self):
        self.withdraw()
        self.tray_icon.visible = True

    def restore_from_tray(self, icon, item):
        self.deiconify()
        self.tray_icon.visible = False

    def exit_program(self, icon, item):
        self.quit()
        self.tray_icon.stop()
        sys.exit()

    def quit(self):
        self.destroy()

    def open_documentation(self, event):
        webbrowser.open("https://docs.klazorix.com/text-expander/getting-started")

    def add_expansion(self):
        dialog = AddExpansionDialog(self)
        self.wait_window(dialog.top)
        self.load_data()
        reload_expansions()


class EditExpansionDialog:
    def __init__(self, parent, trigger, expansion):
        self.parent = parent
        self.top = tk.Toplevel(parent)
        self.top.title("Edit Text Expansion")
        self.top.geometry("300x200")

        self.trigger_label = ttk.Label(self.top, text="Trigger (Max 15 characters):")
        self.trigger_label.pack(pady=5)

        self.trigger_entry = ttk.Entry(self.top)
        self.trigger_entry.insert(0, trigger)
        self.trigger_entry.pack(pady=5)

        self.expansion_label = ttk.Label(self.top, text="Expansion:")
        self.expansion_label.pack(pady=5)

        self.expansion_entry = ttk.Entry(self.top)
        self.expansion_entry.insert(0, expansion)
        self.expansion_entry.pack(pady=5)

        self.save_button = ttk.Button(self.top, text="Save", command=lambda: self.update_expansion(trigger))
        self.save_button.pack(pady=10)

    def update_expansion(self, old_trigger):
        new_trigger = self.trigger_entry.get()
        new_expansion = self.expansion_entry.get()

        if " " in new_trigger:
            messagebox.showerror("Error", "The trigger cannot contain spaces.")
            return

        if new_trigger and new_expansion:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute(
                    "UPDATE text_expansions SET trigger = ?, expansion = ? WHERE trigger = ?",
                    (new_trigger, new_expansion, old_trigger)
                )
                connection.commit()

            self.parent.load_data()
            reload_expansions()
            self.top.destroy()
        else:
            messagebox.showerror("Error", "Please fill both fields.")


class AddExpansionDialog:
    def __init__(self, parent):
        self.parent = parent  # Store parent reference
        self.top = tk.Toplevel(parent)
        self.top.title("Add Text Expansion")
        self.top.geometry("300x200")
        
        self.trigger_label = ttk.Label(self.top, text="Trigger (Max 15 characters):")
        self.trigger_label.pack(pady=5)

        self.trigger_entry = ttk.Entry(self.top)
        self.trigger_entry.pack(pady=5)

        self.expansion_label = ttk.Label(self.top, text="Expansion:")
        self.expansion_label.pack(pady=5)

        self.expansion_entry = ttk.Entry(self.top)
        self.expansion_entry.pack(pady=5)

        self.save_button = ttk.Button(self.top, text="Save", command=self.save_expansion)
        self.save_button.pack(pady=10)

    def save_expansion(self):
        trigger = self.trigger_entry.get()
        expansion = self.expansion_entry.get()

        if trigger and expansion:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("INSERT INTO text_expansions (trigger, expansion) VALUES (?, ?)", (trigger, expansion))
                connection.commit()
            self.parent.load_data()
            reload_expansions()
            self.top.destroy()
        else:
            messagebox.showerror("Error", "Please fill both fields.")


class AddVariableDialog:
    def __init__(self, parent):
        self.top = tk.Toplevel(parent)
        self.top.title("Add Variable")
        self.top.geometry("300x200")

        self.name_label = ttk.Label(self.top, text="Variable Name:")
        self.name_label.pack(pady=5)

        self.name_entry = ttk.Entry(self.top)
        self.name_entry.pack(pady=5)

        self.value_label = ttk.Label(self.top, text="Variable Value:")
        self.value_label.pack(pady=5)

        self.value_entry = ttk.Entry(self.top)
        self.value_entry.pack(pady=5)

        self.save_button = ttk.Button(self.top, text="Save", command=self.save_variable)
        self.save_button.pack(pady=10)

    def save_variable(self):
        name = self.name_entry.get()
        value = self.value_entry.get()

        if name and value:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("INSERT INTO variables (name, value) VALUES (?, ?)", (name, value))
                connection.commit()

                load_data()

            self.top.master.load_data()
            self.top.destroy()
        else:
            messagebox.showerror("Error", "Please fill both fields.")

if __name__ == "__main__":
    typed_buffer = ""
    initialize_db()
    check_updates()
    tray_icon = Icon("TextExpander")
    app = TextExpanderUI(tray_icon)
    threading.Thread(target=create_tray_icon, daemon=True).start()
    keyboard.hook(handle_key_event)
    app.mainloop()