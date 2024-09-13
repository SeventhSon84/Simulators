param (
    [string]$feature
)

switch ($feature) {
    "barcode" { 
        $CONFIG_FILE = "src-tauri/tauri-barcode.conf.json"
    }
    "bna" { 
        $CONFIG_FILE = "src-tauri/tauri-bna.conf.json"
    }
    "assets3" { 
        $CONFIG_FILE = "src-tauri/tauri-assets3.conf.json"
    }
    default {
        Write-Output "Usage: .\build.ps1 [assets1|assets2|assets3]"
        exit 1
    }
}

# Use the selected configuration
Copy-Item $CONFIG_FILE "src-tauri/tauri.conf.json" -Force

# Run Tauri build

# Define the base command
$buildCommand = "cargo tauri build"

# Append the feature flag if provided
if ($feature -ne "default") {
    $buildCommand += " --features=feature-$feature"
}

# Execute the build command
Write-Output "Executing: $buildCommand"
Invoke-Expression $buildCommand
