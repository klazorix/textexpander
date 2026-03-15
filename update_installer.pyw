import requests
import os
import zipfile
import io
import random
import tkinter as tk
from tkinter import ttk, messagebox
import subprocess
import sys
import threading
import psutil

def center_window(win):
    win.update_idletasks()
    width = win.winfo_width()
    height = win.winfo_height()
    x = (win.winfo_screenwidth() // 2) - (width // 2)
    y = (win.winfo_screenheight() // 2) - (height // 2)
    win.geometry(f'{width}x{height}+{x}+{y}')

def update_ui(text, progress=None):
    text_box.configure(state='normal')
    text_box.insert(tk.END, text + '\n')
    text_box.see(tk.END)
    text_box.configure(state='disabled')
    if progress is not None:
        progress_bar['value'] = progress
    root.update_idletasks()

def smooth_progress(current_value, target_value):
    duration = max(20, random.randint(40, 80))  # smoother/faster progress
    increment = (target_value - current_value) / duration
    progress_smoothly(current_value, target_value, increment)

def progress_smoothly(current_value, target_value, increment):
    if current_value < target_value:
        current_value += increment
        if current_value > target_value:
            current_value = target_value
        progress_bar['value'] = current_value
        root.after(20, progress_smoothly, current_value, target_value, increment)
    else:
        update_ui("Installation complete. TextExpander is up-to-date.", 100)
        update_ui("You can now start TextExpander.", 100)
        start_button.config(text="Start TextExpander", command=start_textexpander, state='normal')
        start_button.pack(pady=10)

def terminate_running_process():
    for proc in psutil.process_iter(['pid', 'name']):
        try:
            if proc.info['name'] and proc.info['name'].lower() == 'text_expander.exe':
                proc.terminate()
                proc.wait(timeout=5)
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.TimeoutExpired):
            continue

def download_and_install():
    try:
        update_ui("Retrieving latest version...", 10)
        repo_url = 'https://api.github.com/repos/klazorix/textexpander/releases/latest'
        response = requests.get(repo_url, timeout=15)
        response.raise_for_status()
        data = response.json()

        tag = data.get('tag_name')
        if not tag:
            update_ui("Failed to retrieve latest version tag.", 0)
            return

        tag_clean = tag.lstrip('v')
        zip_url = f'https://github.com/klazorix/textexpander/archive/refs/tags/{tag}.zip'

        update_ui(f"Latest version {tag} found. Downloading zip file...", 20)
        zip_response = requests.get(zip_url, timeout=30)
        zip_response.raise_for_status()

        script_dir = os.path.dirname(os.path.abspath(__file__))
        parent_dir = os.path.abspath(os.path.join(script_dir, '..'))
        exe_path = os.path.join(parent_dir, 'text_expander.exe')

        if os.path.exists(exe_path):
            update_ui("Terminating any running instances of TextExpander...", 30)
            terminate_running_process()
            try:
                os.remove(exe_path)
                update_ui("Previous version stopped and deleted.", 40)
            except Exception as e:
                update_ui(f"Failed to delete old executable: {e}", 40)

        with zipfile.ZipFile(io.BytesIO(zip_response.content)) as zf:
            exe_file_path = f'textexpander-{tag_clean}/text_expander.exe'
            if exe_file_path in zf.namelist():
                update_ui(f"ZIP downloaded. Extracting new files...", 50)
                with zf.open(exe_file_path) as exe_file, open(exe_path, 'wb') as out_file:
                    out_file.write(exe_file.read())
                update_ui(f"File extracted. Installing update...", 70)
                smooth_progress(70, 100)
            else:
                update_ui(f"{exe_file_path} not found in the zip archive.", 100)
                start_button.config(text="Close", command=close_window, state='normal')
                start_button.pack(pady=10)
    except requests.RequestException as e:
        update_ui(f"Network error: {e}")
        messagebox.showerror("Download Error", f"Failed to download update:\n{e}")
        reset_ui()
    except Exception as e:
        update_ui(f"Unexpected error: {e}")
        messagebox.showerror("Error", f"An unexpected error occurred:\n{e}")
        reset_ui()

def reset_ui():
    start_button.config(text="Close", command=close_window, state='normal')
    start_button.pack(pady=10)
    start_download_button.config(state='normal')

def start_download():
    start_download_button.config(state='disabled')
    start_button.config(state='disabled')
    threading.Thread(target=download_and_install, daemon=True).start()

def start_textexpander():
    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        parent_dir = os.path.abspath(os.path.join(script_dir, '..'))
        exe_path = os.path.join(parent_dir, 'text_expander.exe')
        subprocess.Popen(exe_path)
    except Exception as e:
        messagebox.showerror("Error", f"Failed to start TextExpander:\n{e}")
    finally:
        root.quit()

def close_window():
    if progress_bar['value'] < 100:
        if not messagebox.askokcancel("Quit", "Update not complete. Are you sure you want to quit?"):
            return
    root.quit()

root = tk.Tk()
root.title("TextExpander Updater")
root.geometry("550x280")
root.resizable(False, False)

text_box = tk.Text(root, width=70, height=10, state='disabled', wrap='word')
text_box.pack(padx=15, pady=10)

progress_bar = ttk.Progressbar(root, length=500, mode='determinate')
progress_bar.pack(pady=(0, 15))

start_download_button = tk.Button(root, text="Start Download", command=start_download)
start_download_button.pack(pady=10)

start_button = tk.Button(root, text="Close", command=close_window, state='disabled')
start_button.pack_forget()

center_window(root)
root.mainloop()
