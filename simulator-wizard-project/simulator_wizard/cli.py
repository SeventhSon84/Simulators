# simulator_wizard/cli.py

import click
import os
import shutil

# Dynamically set paths relative to the CLI script's location
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
SIMULATORS_PATH = os.path.join(BASE_DIR, "simulator_framework")
ASSETS_PATH = os.path.join(SIMULATORS_PATH, "assets")
PLUGINS_PATH = os.path.join(SIMULATORS_PATH, "plugins")
CARGO_TOML_WORKSPACE = os.path.join(SIMULATORS_PATH, "Cargo.toml")
TAURI_CARGO_TOML = os.path.join(SIMULATORS_PATH, "src-tauri", "Cargo.toml")
MAIN_RS = os.path.join(SIMULATORS_PATH, "src-tauri", "src", "main.rs")
BUILD_SCRIPT = os.path.join(SIMULATORS_PATH, "build.ps1")

@click.group()
def cli():
    """CLI for managing simulators."""
    pass

@cli.command()
@click.option('--name', prompt='Simulator name', help='Name of the new simulator (e.g., bna).')
def create_simulator(name):
    """Create a new simulator with the specified name."""
    create_ui_assets(name)
    create_backend_plugin(name)
    extend_tauri_configuration(name)
    extend_build_script(name)
    click.echo(f'Simulator {name} created successfully.')

def create_ui_assets(simulator_name):
    """Step 1: Create UI assets for the new simulator."""
    src = os.path.join(ASSETS_PATH, "asset-barcode")
    dest = os.path.join(ASSETS_PATH, f"asset-{simulator_name}")
    shutil.copytree(src, dest)
    click.echo(f'UI assets created at {dest}')

def create_backend_plugin(simulator_name):
    """Step 2: Create the backend plugin for the new simulator."""
    src = os.path.join(PLUGINS_PATH, "barcode")
    dest = os.path.join(PLUGINS_PATH, simulator_name)
    shutil.copytree(src, dest)
    click.echo(f'Backend plugin created at {dest}')

    # Modify the Cargo.toml inside the new plugin folder
    modify_cargo_toml(dest, simulator_name)

    # Extend Cargo.toml in the workspace and Tauri app entry point
    extend_cargo_toml_workspace(simulator_name)
    extend_cargo_toml_tauri(simulator_name)

    # Modify main.rs
    extend_main_rs(simulator_name)

def modify_cargo_toml(plugin_path, simulator_name):
    """Modify Cargo.toml of the new plugin."""
    cargo_toml_path = os.path.join(plugin_path, "Cargo.toml")
    with open(cargo_toml_path, "r") as file:
        data = file.read()

    data = data.replace("[package]\nname = \"barcode\"", f"[package]\nname = \"{simulator_name}\"")
    with open(cargo_toml_path, "w") as file:
        file.write(data)
    click.echo(f'Modified Cargo.toml at {cargo_toml_path}')

def extend_cargo_toml_workspace(simulator_name):
    """Extend the workspace's Cargo.toml."""
    with open(CARGO_TOML_WORKSPACE, "r") as file:
        lines = file.readlines()

    # Insert the new plugin path before the closing bracket of the members list
    insert_line = f'    "plugins/{simulator_name}",\n'
    for i, line in enumerate(lines):
        if line.strip() == "]":
            lines.insert(i, insert_line)
            break

    with open(CARGO_TOML_WORKSPACE, "w") as file:
        file.writelines(lines)

    click.echo(f'Extended Cargo.toml of the workspace with plugins/{simulator_name}.')

def extend_cargo_toml_tauri(simulator_name):
    """Extend the Tauri app's Cargo.toml with a new feature and dependency."""
    feature_line = f'feature-{simulator_name} = ["{simulator_name}_plugin"]\n'
    plugin_entry = f'{simulator_name}_plugin = {{ path = "../plugins/{simulator_name}", optional = true }}\n'

    # Read the Cargo.toml file
    with open(TAURI_CARGO_TOML, "r") as file:
        lines = file.readlines()

    # Split lines into sections
    sections = {
        "features": [],
        "dependencies": [],
        "other": []
    }
    current_section = "other"

    # Parse the file into sections
    for line in lines:
        stripped_line = line.strip()
        if stripped_line == "[features]":
            current_section = "features"
            sections[current_section].append(line)
        elif stripped_line == "[dependencies]":
            current_section = "dependencies"
            sections[current_section].append(line)
        elif stripped_line == "" and current_section in ["features", "dependencies"]:
            # Skip empty lines that follow section headers
            continue
        else:
            sections[current_section].append(line)

    # Add feature and dependency if they don't already exist
    if feature_line not in sections["features"]:
        sections["features"].append(feature_line)

    if plugin_entry not in sections["dependencies"]:
        sections["dependencies"].append(plugin_entry)

    # Write the updated content back to the file
    with open(TAURI_CARGO_TOML, "w") as file:
        for section in ["other", "dependencies", "features"]:
            if section in sections:
                # Avoid adding extra newlines between sections
                for line in sections[section]:
                    file.write(line)

    click.echo(f'Extended Tauri app Cargo.toml with {simulator_name}_plugin and feature-{simulator_name}.')




def extend_main_rs(plugin_name):
    """Extend the main.rs file with a new plugin feature."""
    # Define new lines to be added
    new_feature_import = f'#[cfg(feature = "feature-{plugin_name}")]\nuse {plugin_name}_plugin::{plugin_name.capitalize()}Plugin;\n'
    new_feature_type = f'#[cfg(feature = "feature-{plugin_name}")]\ntype SelectedPlugin = {plugin_name.capitalize()}Plugin;\n'

    # Read the main.rs file
    with open(MAIN_RS, "r") as file:
        lines = file.readlines()

    # Variables to track the index where to add the new lines
    last_selected_plugin_index = -1

    # Find the index of the last `type SelectedPlugin =` line
    for i, line in enumerate(lines):
        if 'type SelectedPlugin =' in line:
            last_selected_plugin_index = i

    # Write new content to a temporary file
    temp_file_path = MAIN_RS + ".tmp"
    with open(temp_file_path, "w") as temp_file:
        for i, line in enumerate(lines):
            temp_file.write(line)
            # Insert new feature lines after the last `type SelectedPlugin =` line
            if i == last_selected_plugin_index:
                temp_file.write("\n")
                temp_file.write(new_feature_import)
                temp_file.write(new_feature_type)
                temp_file.write("\n")

    # Replace the original main.rs with the updated one
    shutil.move(temp_file_path, MAIN_RS)

    click.echo(f'Extended main.rs with {plugin_name}_plugin.')






def extend_tauri_configuration(simulator_name):
    """Step 3: Extend the Tauri configuration."""
    src = os.path.join(SIMULATORS_PATH, "src-tauri", "tauri-barcode.conf.json")
    dest = os.path.join(SIMULATORS_PATH, "src-tauri", f"tauri-{simulator_name}.conf.json")
    shutil.copyfile(src, dest)
    click.echo(f'Created new Tauri configuration {dest}')

def extend_build_script(simulator_name):
    """Extend the build script to include the new Tauri configuration file and build command."""
    # Define the source and destination paths for the configuration file
    tauri_conf_src = os.path.join(SIMULATORS_PATH, f"src-tauri/tauri-{simulator_name}.conf.json")
    tauri_conf_dest = os.path.join(SIMULATORS_PATH, "src-tauri", f"tauri-{simulator_name}.conf.json")

    # Check if the source and destination are different
    if not os.path.exists(tauri_conf_dest) or not os.path.samefile(tauri_conf_src, tauri_conf_dest):
        # Copy the new configuration file to the src-tauri folder
        shutil.copyfile(tauri_conf_src, tauri_conf_dest)
        click.echo(f'Copied Tauri configuration to {tauri_conf_dest}')
    else:
        click.echo(f'Tauri configuration file already exists at {tauri_conf_dest}')

    # Define the build command with the feature flag
    build_command = f'cargo tauri build --features feature-{simulator_name}'

    # Read the existing build.ps1 file
    with open(BUILD_SCRIPT, "r") as file:
        lines = file.readlines()

    # Check if the feature already exists in the build.ps1 script
    feature_exists = False
    for line in lines:
        if f'feature-{simulator_name}' in line:
            feature_exists = True
            break

    if not feature_exists:
        # Insert the new feature case into the switch statement and update the usage message
        new_case = f'    "{simulator_name}" {{ \n        $CONFIG_FILE = "src-tauri/tauri-{simulator_name}.conf.json"\n    }}\n'
        usage_message = '        Write-Output "Usage: .\\build.ps1 [barcode|bna|assets3|card|{simulator_name}]"'

        # Find where to insert the new case
        for i, line in enumerate(lines):
            if "default" in line:
                # Insert the new case before the default
                lines.insert(i, new_case)
                # Update the usage message in the default block
                lines[i + 1] = lines[i + 1].replace("Usage: .\\build.ps1", usage_message)
                break

        # Write the updated content back to build.ps1
        with open(BUILD_SCRIPT, "w") as file:
            file.writelines(lines)

        click.echo(f'Updated build.ps1 with new feature: {simulator_name}')

    click.echo(f'Extended build script with {simulator_name} configuration.')



if __name__ == '__main__':
    cli()
