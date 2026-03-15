import os
import sys
import subprocess
import threading
import time
import datetime
import sqlite3
import webbrowser
import logging

try:
    import tkinter as tk
    from tkinter import ttk, messagebox
except Exception:
    raise RuntimeError("Tkinter is required but not available in this environment.")

try:
    import psutil
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "psutil"])
    import psutil

try:
    import pyperclip
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyperclip"])
    import pyperclip

try:
    import keyboard
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "keyboard"])
    import keyboard

try:
    import pyautogui
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pyautogui"])
    import pyautogui

try:
    from pystray import Icon, MenuItem, Menu
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pystray"])
    from pystray import Icon, MenuItem, Menu

try:
    from PIL import Image
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "Pillow"])
    from PIL import Image

try:
    import requests
    import zipfile
    import io
except Exception:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
    import requests
    import zipfile
    import io

logging.basicConfig(level=logging.INFO, filename="text_expander.log", filemode="a",
                    format="%(asctime)s %(levelname)s %(message)s")

if hasattr(sys, '_MEIPASS'):
    SCRIPT_DIR = sys._MEIPASS
else:
    SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

DATA_FILE = os.path.join(SCRIPT_DIR, "text_expander_data.db")
ICON_PATH = os.path.join(SCRIPT_DIR, "icon.png")
VERSION = "v3.0.0"

if os.path.exists(ICON_PATH):
    try:
        TRAY_ICON = Image.open(ICON_PATH)
    except Exception:
        TRAY_ICON = Image.new("RGBA", (64, 64), (51, 57, 198, 255))
else:
    TRAY_ICON = Image.new("RGBA", (64, 64), (51, 57, 198, 255))


def safe_iter_processes():
    for proc in psutil.process_iter(['name', 'cmdline']):
        try:
            yield proc
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue


for proc in safe_iter_processes():
    try:
        cmdline = proc.info.get('cmdline', [])
        name = proc.info.get('name', '')
        if name and 'python' in name.lower() and cmdline:
            for part in cmdline:
                try:
                    if 'text_expander' in str(part).lower():
                        logging.info(f"Text Expander detected running process: {proc.pid} {proc.info}")
                        break
                except Exception:
                    continue
    except Exception:
        continue


def initialize_db():
    try:
        conn = sqlite3.connect(DATA_FILE)
        cursor = conn.cursor()
        cursor.execute(''' 
        CREATE TABLE IF NOT EXISTS text_expansions ( 
            trigger TEXT PRIMARY KEY, 
            expansion TEXT 
        )''')
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )''')
        conn.commit()
        conn.close()
    except Exception:
        logging.exception("Failed to initialize database")
        raise


def get_setting(key, default=None):
    try:
        with sqlite3.connect(DATA_FILE) as conn:
            cur = conn.cursor()
            cur.execute("SELECT value FROM settings WHERE key = ?", (key,))
            row = cur.fetchone()
            if row:
                val = row[0]
                if val.lower() in ("true", "1", "yes"):
                    return True
                elif val.lower() in ("false", "0", "no"):
                    return False
                return val
    except Exception:
        logging.exception("Failed to get setting")
    return default


def set_setting(key, value):
    try:
        with sqlite3.connect(DATA_FILE) as conn:
            cur = conn.cursor()
            cur.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)", (key, str(value)))
            conn.commit()
    except Exception:
        logging.exception("Failed to set setting")


def load_expansions():
    try:
        if not os.path.exists(DATA_FILE):
            initialize_db()
        with sqlite3.connect(DATA_FILE) as connection:
            cursor = connection.cursor()
            cursor.execute("SELECT trigger, expansion FROM text_expansions")
            return dict(cursor.fetchall())
    except Exception:
        logging.exception("Failed to load expansions")
        return {}


text_expansions = load_expansions()


class PyUtils:
    @staticmethod
    def action_handler(action: str):
        try:
            if not action:
                return
            action = action.strip()
            if action.startswith("open:"):
                path = action.split("open:", 1)[1].strip()
                if path:
                    webbrowser.open(path)
            elif action.startswith("run:"):
                program = action.split("run:", 1)[1].strip()
                if program:
                    subprocess.Popen(program, shell=True)
            elif action.startswith("cmd:"):
                cmd = action.split("cmd:", 1)[1].strip()
                if cmd:
                    subprocess.Popen(cmd, shell=True)
            elif action == "clipboard":
                pyperclip.copy("")
            else:
                logging.info(f"Unknown action requested: {action}")
        except Exception:
            logging.exception("Error in action_handler")


pyutils = PyUtils()


def replace_placeholders(text):
    try:
        text = text.replace("{date}", datetime.datetime.now().strftime("%d/%m/%Y"))
        text = text.replace("{time}", datetime.datetime.now().strftime("%H:%M"))
        try:
            text = text.replace("{clipboard}", pyperclip.paste())
        except Exception:
            text = text.replace("{clipboard}", "")
        return text
    except Exception:
        logging.exception("Placeholder replacement failed")
        return text


typed_buffer = ""
typed_lock = threading.Lock()

# Global setting variable (to be loaded and updated)
USE_BACKSPACE_TO_REMOVE_TRIGGER = True


def handle_key_event(event):
    global typed_buffer, USE_BACKSPACE_TO_REMOVE_TRIGGER
    try:
        if event.event_type != "down":
            return
        name = getattr(event, "name", "")
        if not name:
            return

        allowed_chars = "abcdefghijklmnopqrstuvwxyz0123456789-_=+[]\\;',.`~!@#$%^&*(){}|:\"<>?"
        if len(name) == 1 and name.lower() in allowed_chars:
            with typed_lock:
                typed_buffer += name
            return
        elif name in ("space", "enter"):
            with typed_lock:
                buffer_copy = typed_buffer
                typed_buffer = ""
            if buffer_copy:
                expansion = text_expansions.get(buffer_copy)
                if expansion is not None:
                    expansion = replace_placeholders(expansion)
                    try:
                        if USE_BACKSPACE_TO_REMOVE_TRIGGER:
                            backspaces_needed = len(buffer_copy) + 1
                            pyautogui.press('backspace', presses=backspaces_needed, interval=0.01)
                        else:
                            pyautogui.hotkey('ctrl', 'a')
                            time.sleep(0.05)
                            pyautogui.press('backspace')
                    except Exception:
                        pass
                    try:
                        keyboard.write(expansion)
                    except Exception:
                        try:
                            pyautogui.write(expansion)
                        except Exception:
                            logging.exception("Failed to write expansion")
            return
        elif name == "backspace":
            with typed_lock:
                typed_buffer = typed_buffer[:-1] if typed_buffer else ""
            return
        else:
            with typed_lock:
                typed_buffer = ""
    except Exception:
        logging.exception("Unexpected error in handle_key_event")


def reload_expansions():
    global text_expansions
    try:
        text_expansions = load_expansions()
        logging.info(f"Reloaded {len(text_expansions)} expansions.")
    except Exception:
        logging.exception("Failed to reload expansions")
        text_expansions = {}


def create_tray_icon(app_ref):
    try:
        menu = Menu(
            MenuItem("Open", lambda icon, item: app_ref.restore_from_tray(icon, item)),
            MenuItem("Exit", lambda icon, item: app_ref.exit_program(icon, item))
        )
        tray = Icon("TextExpander", TRAY_ICON, "TextExpander by @klazorix", menu=menu)
        tray.run()
    except Exception:
        logging.exception("Tray icon failed")


class TextExpanderUI(tk.Tk):
    def __init__(self):
        super().__init__()
        global USE_BACKSPACE_TO_REMOVE_TRIGGER

        self.tray_icon = None
        self.withdraw()
        self.title(f"TextExpander {VERSION}")
        self.geometry("650x480")
        self.resizable(False, False)
        self.protocol("WM_DELETE_WINDOW", self.minimize_to_tray)
        self.create_widgets()
        self.load_data()

        # Load settings
        USE_BACKSPACE_TO_REMOVE_TRIGGER = get_setting("use_backspace_to_remove_trigger", True)
        self.use_backspace_var.set(USE_BACKSPACE_TO_REMOVE_TRIGGER)

        try:
            messagebox.showinfo(
                f"TextExpander {VERSION} - Notifications",
                f"TextExpander is now running!\n\nUse your system tray to exit the program or update expansions/variables."
            )
        except Exception:
            pass

    def create_widgets(self):
        default_font = ("Segoe UI", 11)
        self.option_add("*Font", default_font)

        self.tab_control = ttk.Notebook(self)
        self.tab_control.pack(fill=tk.BOTH, expand=True, padx=12, pady=12)

        self.tab_expansion = ttk.Frame(self.tab_control)
        self.tab_about = ttk.Frame(self.tab_control)
        self.tab_settings = ttk.Frame(self.tab_control)

        self.tab_control.add(self.tab_expansion, text="Text Expansions")
        self.tab_control.add(self.tab_about, text="About")
        self.tab_control.add(self.tab_settings, text="Settings")

        # Text Expansions tab
        self.text_expansion_tree = ttk.Treeview(
            self.tab_expansion, columns=("Trigger", "Expansion"), show="headings", height=15
        )
        self.text_expansion_tree.heading("Trigger", text="Trigger")
        self.text_expansion_tree.heading("Expansion", text="Expansion")
        self.text_expansion_tree.column("Trigger", width=160, anchor="w")
        self.text_expansion_tree.column("Expansion", width=460, anchor="w")
        self.text_expansion_tree.pack(fill=tk.BOTH, expand=True)

        exp_button_frame = ttk.Frame(self.tab_expansion)
        exp_button_frame.pack(fill=tk.X, pady=8)
        self.edit_button = ttk.Button(exp_button_frame, text="Edit Selected", command=self.edit_expansion_selected)
        self.edit_button.pack(side=tk.LEFT, padx=6)
        self.delete_button = ttk.Button(exp_button_frame, text="Delete Selected", command=self.delete_expansion_selected)
        self.delete_button.pack(side=tk.LEFT, padx=6)
        self.add_expansion_button = ttk.Button(exp_button_frame, text="Add New Expansion", command=self.add_expansion)
        self.add_expansion_button.pack(side=tk.RIGHT, padx=6)

        # About tab
        about_text = (
            "Thank you for using TextExpander.\n\n"
            "This tool helps you quickly expand typed triggers into longer text snippets.\n\n"
            "If you encounter any bugs or want to request features, please visit the GitHub page:\n"
            "https://github.com/klazorix/textexpander\n\n"
            "If you require assistance with TextExpander, please visit the Docs page:\n"
            "https://docs.klazorix.com/textexpander\n\n"
            "Developed by @klazorix\n"
            f"Running TextExpander {VERSION}"
        )
        self.help_text = tk.Label(
            self.tab_about,
            text=about_text,
            justify="center",
            wraplength=580,
            padx=15,
            pady=15
        )
        self.help_text.pack(fill=tk.BOTH, expand=True)

        # Settings tab
        self.use_backspace_var = tk.BooleanVar(value=True)
        self.use_backspace_var.set(get_setting("use_backspace_to_remove_trigger", True))
        chk = ttk.Checkbutton(
            self.tab_settings,
            text="Use backspace to remove trigger",
            variable=self.use_backspace_var,
            command=self.toggle_backspace_setting
        )
        chk.pack(pady=20, padx=20, anchor="w")

    def toggle_backspace_setting(self):
        global USE_BACKSPACE_TO_REMOVE_TRIGGER
        USE_BACKSPACE_TO_REMOVE_TRIGGER = self.use_backspace_var.get()
        set_setting("use_backspace_to_remove_trigger", USE_BACKSPACE_TO_REMOVE_TRIGGER)
        logging.info(f"Setting 'use_backspace_to_remove_trigger' set to {USE_BACKSPACE_TO_REMOVE_TRIGGER}")

    def load_data(self):
        for row in self.text_expansion_tree.get_children():
            self.text_expansion_tree.delete(row)
        try:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("SELECT trigger, expansion FROM text_expansions ORDER BY trigger ASC")
                rows = cursor.fetchall()
                for row in rows:
                    self.text_expansion_tree.insert("", tk.END, values=row)
        except Exception:
            logging.exception("Failed to load UI data")

    def edit_expansion_selected(self):
        selected_item = self.text_expansion_tree.selection()
        if not selected_item:
            messagebox.showinfo("Error", "Select an expansion to edit.")
            return
        values = self.text_expansion_tree.item(selected_item, "values")
        if not values or len(values) < 2:
            messagebox.showerror("Error", "Selected item is invalid.")
            return
        trigger, expansion = values
        dialog = EditExpansionDialog(self, trigger, expansion)
        self.wait_window(dialog.top)
        self.load_data()
        reload_expansions()

    def delete_expansion_selected(self):
        selected_item = self.text_expansion_tree.selection()
        if not selected_item:
            messagebox.showerror("Error", "Please select an expansion to delete.")
            return
        values = self.text_expansion_tree.item(selected_item, "values")
        if not values:
            messagebox.showerror("Error", "Selected item is invalid.")
            return
        trigger = values[0]
        if messagebox.askyesno("Delete", f"Are you sure you want to delete the expansion for '{trigger}'?"):
            try:
                with sqlite3.connect(DATA_FILE) as connection:
                    cursor = connection.cursor()
                    cursor.execute("DELETE FROM text_expansions WHERE trigger = ?", (trigger,))
                    connection.commit()
            except Exception:
                logging.exception("Failed to delete expansion")
            self.load_data()
            reload_expansions()

    def add_expansion(self):
        dialog = AddExpansionDialog(self)
        self.wait_window(dialog.top)
        self.load_data()
        reload_expansions()

    def minimize_to_tray(self):
        self.withdraw()

    def restore_from_tray(self, icon, item):
        self.after(0, self.deiconify)

    def exit_program(self, icon, item):
        icon.stop()
        self.destroy()
        sys.exit(0)


class EditExpansionDialog:
    def __init__(self, parent, trigger, expansion):
        self.parent = parent
        self.top = tk.Toplevel(parent)
        self.top.title("Edit Expansion")
        self.top.geometry("400x300")
        self.top.resizable(False, False)
        self.top.grab_set()

        ttk.Label(self.top, text="Trigger:").pack(pady=(12, 4), anchor="w", padx=12)
        self.trigger_entry = ttk.Entry(self.top)
        self.trigger_entry.insert(0, trigger)
        self.trigger_entry.pack(fill=tk.X, padx=12)

        ttk.Label(self.top, text="Expansion:").pack(pady=(12, 4), anchor="w", padx=12)
        self.expansion_text = tk.Text(self.top, height=7, wrap=tk.WORD)
        self.expansion_text.insert(tk.END, expansion)
        self.expansion_text.pack(fill=tk.BOTH, expand=True, padx=12)

        save_btn = ttk.Button(self.top, text="Save", command=self.save)
        save_btn.pack(pady=12)

        self.original_trigger = trigger

    def save(self):
        trigger = self.trigger_entry.get().strip()
        expansion = self.expansion_text.get("1.0", tk.END).strip()
        if not trigger or not expansion:
            messagebox.showerror("Error", "Both Trigger and Expansion are required.")
            return
        try:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                if trigger != self.original_trigger:
                    cursor.execute("SELECT trigger FROM text_expansions WHERE trigger = ?", (trigger,))
                    if cursor.fetchone():
                        messagebox.showerror("Error", f"Trigger '{trigger}' already exists.")
                        return
                cursor.execute(
                    "UPDATE text_expansions SET trigger = ?, expansion = ? WHERE trigger = ?",
                    (trigger, expansion, self.original_trigger)
                )
                connection.commit()
        except Exception:
            logging.exception("Failed to update expansion")
            messagebox.showerror("Error", "Failed to save expansion.")
            return
        self.parent.load_data()
        self.top.destroy()


class AddExpansionDialog:
    def __init__(self, parent):
        self.parent = parent
        self.top = tk.Toplevel(parent)
        self.top.title("Add Expansion")
        self.top.geometry("400x300")
        self.top.resizable(False, False)
        self.top.grab_set()

        ttk.Label(self.top, text="Trigger:").pack(pady=(12, 4), anchor="w", padx=12)
        self.trigger_entry = ttk.Entry(self.top)
        self.trigger_entry.pack(fill=tk.X, padx=12)

        ttk.Label(self.top, text="Expansion:").pack(pady=(12, 4), anchor="w", padx=12)
        self.expansion_text = tk.Text(self.top, height=7, wrap=tk.WORD)
        self.expansion_text.pack(fill=tk.BOTH, expand=True, padx=12)

        save_btn = ttk.Button(self.top, text="Save", command=self.save)
        save_btn.pack(pady=12)

    def save(self):
        trigger = self.trigger_entry.get().strip()
        expansion = self.expansion_text.get("1.0", tk.END).strip()
        if not trigger or not expansion:
            messagebox.showerror("Error", "Both Trigger and Expansion are required.")
            return
        try:
            with sqlite3.connect(DATA_FILE) as connection:
                cursor = connection.cursor()
                cursor.execute("SELECT trigger FROM text_expansions WHERE trigger = ?", (trigger,))
                if cursor.fetchone():
                    messagebox.showerror("Error", f"Trigger '{trigger}' already exists.")
                    return
                cursor.execute("INSERT INTO text_expansions (trigger, expansion) VALUES (?, ?)",
                               (trigger, expansion))
                connection.commit()
        except Exception:
            logging.exception("Failed to add expansion")
            messagebox.showerror("Error", "Failed to save expansion.")
            return
        self.parent.load_data()
        self.top.destroy()


def check_updates(silent=False):
    try:
        repo_api = "https://api.github.com/repos/klazorix/textexpander/releases/latest"
        response = requests.get(repo_api, timeout=8)
        if response.status_code != 200:
            if not silent:
                logging.warning("Update check returned non-200 status.")
            return
        data = response.json()
        latest = data.get("tag_name", "")
        if latest and VERSION != latest:
            answer = messagebox.askyesno(
                f"TextExpander {VERSION} - Update Available",
                f"TextExpander {latest} is now available!\nWould you like to upgrade?"
            )
            if answer:
                try:
                    tag = data.get("tag_name")
                    zip_url = f'https://github.com/klazorix/textexpander/archive/refs/tags/{tag}.zip'
                    zip_data = requests.get(zip_url, timeout=15)
                    with zipfile.ZipFile(io.BytesIO(zip_data.content)) as zf:
                        exe_file = f'textexpander-{tag.lstrip("v")}/_internal/update_installer.pyw'
                        if exe_file in zf.namelist():
                            internal_dir = os.path.join(SCRIPT_DIR, '_internal')
                            os.makedirs(internal_dir, exist_ok=True)
                            exe_path = os.path.join(internal_dir, 'update_installer.pyw')
                            with zf.open(exe_file) as exe, open(exe_path, 'wb') as out:
                                out.write(exe.read())
                            try:
                                subprocess.Popen([sys.executable, exe_path], shell=False)
                                messagebox.showinfo("Updater", "Updater started.")
                            except Exception as e:
                                messagebox.showerror("Error", f"Failed to start updater: {e}")
                except Exception as e:
                    messagebox.showerror("Error", f"Update failed: {e}")
    except requests.exceptions.RequestException as e:
        logging.warning(f"Update check failed: {e}")
    except Exception as e:
        logging.exception("Unexpected error during update check")


def main():
    global USE_BACKSPACE_TO_REMOVE_TRIGGER
    try:
        initialize_db()
        reload_expansions()
        USE_BACKSPACE_TO_REMOVE_TRIGGER = get_setting("use_backspace_to_remove_trigger", True)
        check_updates(silent=True)

        app = TextExpanderUI()
        tray_thread = threading.Thread(target=create_tray_icon, args=(app,), daemon=True)
        tray_thread.start()

        keyboard.hook(handle_key_event)

        app.mainloop()
    except Exception:
        logging.exception("Unhandled exception in main")


if __name__ == "__main__":
    main()
