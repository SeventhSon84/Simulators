# simulator_wizard/wizard.py

import questionary
import subprocess
import os

# Determine the base directory and the path to the CLI script
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
CLI_PATH = os.path.join(BASE_DIR, "simulator_wizard", "cli.py")

def run_wizard():
    """Wizard to interactively create a new simulator."""
    # Question to ask the user
    simulator_name = questionary.text("What's the name of the simulator?").ask()

    if not simulator_name:
        print("No simulator name provided. Exiting.")
        return

    # Call the CLI command to create the simulator
    result = subprocess.run(
        ["poetry", "run", "python", CLI_PATH, "create-simulator", f"--name={simulator_name}"],
        capture_output=True,
        text=True,
        cwd=BASE_DIR,  # Run the command from the project directory
    )

    # Display the output from the CLI
    print(result.stdout)
    if result.stderr:
        print("Error:", result.stderr)

if __name__ == "__main__":
    run_wizard()
