import requests
import os
import zipfile
import io
import random
import tkinter as tk
from tkinter import ttk
import subprocess
import sys

try:
    import psutil
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "psutil"])
    import psutil

def update_ui(text, progress=None):
    text_box.insert(tk.END, text + '\n')
    text_box.yview(tk.END)
    if progress is not None:
        progress_bar['value'] = progress
    root.update_idletasks()

def smooth_progress(current_value, target_value):
    duration = random.randint(50, 100)
    increment = (target_value - current_value) / duration
    progress_smoothly(current_value, target_value, increment)

def progress_smoothly(current_value, target_value, increment):
    if current_value < target_value:
        current_value += increment
        if current_value > target_value:
            current_value = target_value
        progress_bar['value'] = current_value
        root.after(10, progress_smoothly, current_value, target_value, increment)
    else:
        update_ui("Installation complete. TextExpander is up-to-date.", 100)
        update_ui("You can now start TextExpander.", 100)
        start_button.config(text="Start TextExpander", command=start_textexpander)
        start_button.pack(pady=10)

def terminate_running_process():
    exe_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'text_expander.exe')
    for proc in psutil.process_iter(['pid', 'name']):
        if proc.info['name'] == 'text_expander.exe':
            proc.terminate()

def start_download():
    start_download_button.pack_forget()
    update_ui("Retrieving latest version...", 10)
    repo_url = 'https://api.github.com/repos/klazorix/textexpander/releases/latest'
    response = requests.get(repo_url)
    data = response.json()

    tag = data['tag_name'].lstrip('v')
    zip_url = f'https://github.com/klazorix/textexpander/archive/refs/tags/{data["tag_name"]}.zip'

    zip_data = requests.get(zip_url)
    update_ui("Latest version retrieved. Downloading zip file...", 20)

    script_dir = os.path.dirname(os.path.abspath(__file__))
    exe_path = os.path.join(os.path.dirname(script_dir), 'text_expander.exe')

    if os.path.exists(exe_path):
        update_ui("Terminating any running instances of TextExpander...", 30)
        terminate_running_process()
        os.remove(exe_path)
        update_ui("Previous version stopped and deleted.", 40)

    with zipfile.ZipFile(io.BytesIO(zip_data.content)) as zf:
        exe_file = f'textexpander-{tag}/text_expander.exe'
        if exe_file in zf.namelist():
            update_ui(f"ZIP downloaded. Extracting new files...", 50)
            exe_path = os.path.join(os.path.dirname(script_dir), 'text_expander.exe')
            with zf.open(exe_file) as exe:
                with open(exe_path, 'wb') as f:
                    f.write(exe.read())
            update_ui(f"File extracted. Installing update...", 70)
            smooth_progress(70, 100)
        else:
            update_ui(f"{exe_file} not found inside the zip archive.", 100)
            update_ui("You can now close the window.", 100)
            start_button.config(text="Start TextExpander", command=start_textexpander)
            start_button.pack(pady=10)

def start_textexpander():
    exe_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../text_expander.exe')
    subprocess.Popen(exe_path)
    root.quit()

def close_window():
    root.quit()

root = tk.Tk()
root.title("File Download and Extraction")

text_box = tk.Text(root, width=80, height=10)
text_box.pack(pady=10)

progress_bar = ttk.Progressbar(root, length=400, mode='determinate')
progress_bar.pack(pady=10)

start_download_button = tk.Button(root, text="Start Download", command=start_download)
start_download_button.pack(pady=10)

start_button = tk.Button(root, text="Close", command=close_window)
start_button.pack_forget()

root.mainloop()