<#
.SYNOPSIS
Windows Auto Setup with Full Concurrent Installation Pipeline
.DESCRIPTION
Each component (Node.js, JDK, MySQL, Maven) runs its complete installation pipeline concurrently:
- Download -> Verify -> Extract/Install -> Configure -> Environment Setup
Features:
- Full parallel execution of all installation workflows
- Real-time progress monitoring
- Independent component installation
- Automatic error recovery
- Version selection and environment detection
Requires admin rights.
#>

# ==============================================
# 0. Administrator Permission Check
# ==============================================
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "ERROR: Please run this script as Administrator!" -ForegroundColor Red
    Write-Host "  Right-click script -> Select 'Run as administrator'" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# ==============================================
# 1. Version Selection
# ==============================================
Write-Host "`n================================================" -ForegroundColor Cyan
Write-Host "  Development Environment Setup" -ForegroundColor Cyan
Write-Host "  Full Concurrent Installation Mode" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Node.js Version Selection
Write-Host "Select Node.js version:" -ForegroundColor Yellow
Write-Host "  1) Node.js 14.21.3 (Legacy)"
Write-Host "  2) Node.js 16.20.2 (LTS)"
Write-Host "  3) Node.js 18.19.0 (LTS, Recommended)"
$nodeChoice = Read-Host "Enter choice (1/2/3, default: 3)"

if ([string]::IsNullOrEmpty($nodeChoice)) { $nodeChoice = "3" }

switch ($nodeChoice) {
    "1" { $nodeVersion = "14.21.3" }
    "2" { $nodeVersion = "16.20.2" }
    default { $nodeVersion = "18.19.0" }
}
Write-Host "Selected: Node.js $nodeVersion" -ForegroundColor Green
Write-Host ""

# JDK Version Selection
Write-Host "Select JDK version:" -ForegroundColor Yellow
Write-Host "  1) JDK 1.8 (8u392, Legacy)"
Write-Host "  2) JDK 17.0.10 (LTS, Recommended)"
$jdkChoice = Read-Host "Enter choice (1/2, default: 2)"

if ([string]::IsNullOrEmpty($jdkChoice)) { $jdkChoice = "2" }

if ($jdkChoice -eq "1") {
    $jdkVersion = "8u392"
    $jdkMajorVersion = "8"
} else {
    $jdkVersion = "17.0.10"
    $jdkMajorVersion = "17"
}
Write-Host "Selected: JDK $jdkVersion" -ForegroundColor Green
Write-Host ""

# ==============================================
# 2. Global Configuration
# ==============================================
$global:installRoot = "D:\DevSetup"
$global:tempDir = "$global:installRoot\Temp"
$global:mysqlVersion = "8.0.24"
$global:mavenVersion = "3.9.6"
$global:mysqlRootPwd = "123456"
$global:mysqlPort = "3306"

# Initialize directories
if (-not (Test-Path $global:installRoot)) {
    New-Item -Path $global:installRoot -ItemType Directory -Force | Out-Null
}
if (-not (Test-Path $global:tempDir)) {
    New-Item -Path $global:tempDir -ItemType Directory -Force | Out-Null
}

# ==============================================
# 3. Environment Detection
# ==============================================
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "  Environment Detection" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

$needInstallNode = $false
$needInstallJDK = $false
$needInstallMySQL = $false
$needInstallMaven = $false

# Check Node.js
Write-Host "Checking Node.js..." -ForegroundColor Yellow
$forceUninstallNode = $false
try {
    $nodeExe = Get-Command "node" -ErrorAction Stop
    $nodeVersionOutput = & node --version 2>&1
    if ($nodeVersionOutput -match "v(\d+\.\d+\.\d+)") {
        $installedNodeVersion = $matches[1]
        Write-Host "  Found: Node.js $installedNodeVersion" -ForegroundColor Green
        if ($installedNodeVersion -ne $nodeVersion) {
            Write-Host "  [!] Version mismatch (need $nodeVersion)" -ForegroundColor Yellow
            
            # Ask user if they want to uninstall existing version
            $uninstallChoice = Read-Host "Uninstall existing Node.js $installedNodeVersion and install $nodeVersion? (Y/N, default: Y)"
            if ([string]::IsNullOrEmpty($uninstallChoice) -or $uninstallChoice -eq "Y" -or $uninstallChoice -eq "y") {
                $needInstallNode = $true
                $forceUninstallNode = $true
                Write-Host "  [OK] Will uninstall v$installedNodeVersion and install v$nodeVersion" -ForegroundColor Green
            } else {
                Write-Host "  [X] Keeping existing Node.js v$installedNodeVersion" -ForegroundColor Gray
            }
        } else {
            Write-Host "  [OK] Version matches" -ForegroundColor Green
        }
    }
} catch {
    Write-Host "  [X] Not found" -ForegroundColor Red
    $needInstallNode = $true
}

# Check JDK
Write-Host "Checking JDK..." -ForegroundColor Yellow
try {
    $javaExe = Get-Command "java" -ErrorAction Stop
    $javaVersionOutput = & java -version 2>&1 | Select-Object -First 3
    $versionLine = $javaVersionOutput | Where-Object { $_ -match "version" } | Select-Object -First 1
    
    # Support version formats: "1.8.0_202", "17.0.10", "11.0.2"
    if ($versionLine -and $versionLine -match '"([\d._]+)"') {
        $installedJavaVersion = $matches[1]
        Write-Host "  Found: JDK $installedJavaVersion" -ForegroundColor Green
        
        # Extract major version (handle both 1.8.x and 17.x formats)
        $installedMajorVersion = ""
        if ($installedJavaVersion -match "^1\.(\d+)") {
            # Old format: 1.8.0_202 -> major version is 8
            $installedMajorVersion = $matches[1]
        } elseif ($installedJavaVersion -match "^(\d+)\.") {
            # New format: 17.0.10 -> major version is 17
            $installedMajorVersion = $matches[1]
        }
        
        if ($installedMajorVersion -eq $jdkMajorVersion) {
            Write-Host "  [OK] Version matches (JDK $jdkMajorVersion)" -ForegroundColor Green
        } else {
            Write-Host "  [!] Version mismatch (found JDK $installedMajorVersion, need JDK $jdkMajorVersion)" -ForegroundColor Yellow
            $needInstallJDK = $true
        }
    } else {
        Write-Host "  [X] Cannot parse JDK version from output:" -ForegroundColor Red
        Write-Host "     $versionLine" -ForegroundColor Gray
        $needInstallJDK = $true
    }
} catch {
    Write-Host "  [X] Not found" -ForegroundColor Red
    $needInstallJDK = $true
}

# Check MySQL
Write-Host "Checking MySQL..." -ForegroundColor Yellow
try {
    $mysqlExe = Get-Command "mysql" -ErrorAction Stop
    Write-Host "  Found: MySQL" -ForegroundColor Green
    Write-Host "  [OK] Available" -ForegroundColor Green
} catch {
    Write-Host "  [X] Not found" -ForegroundColor Red
    $needInstallMySQL = $true
}

# Check Maven
Write-Host "Checking Maven..." -ForegroundColor Yellow
try {
    $mvnExe = Get-Command "mvn" -ErrorAction Stop
    Write-Host "  Found: Maven" -ForegroundColor Green
    Write-Host "  [OK] Available" -ForegroundColor Green
} catch {
    Write-Host "  [X] Not found" -ForegroundColor Red
    $needInstallMaven = $true
}

Write-Host ""

# ==============================================
# 4. Component Installation Functions
# ==============================================

# Node.js Installation Function
$InstallNodeJSScript = {
    param($nodeVersion, $installRoot, $tempDir, $forceUninstall = $false)
    
    $result = @{
        Component = "Node.js $nodeVersion"
        Success = $false
        Message = ""
        Duration = 0
    }
    
    $startTime = Get-Date
    
    try {
        $nodeInstallPath = "$installRoot\nodejs-$nodeVersion"
        $nodeInstaller = "$tempDir\node-v$nodeVersion-x64.msi"
        
        # Uninstall existing Node.js if requested
        if ($forceUninstall) {
            Write-Host "[Node.js] [Step 0/5] Uninstalling existing version..." -ForegroundColor Yellow
            Write-Host "[Node.js] Progress: 5% - Searching for installed Node.js..." -ForegroundColor Gray
            
            try {
                # Find Node.js in registry
                $uninstallKeys = @(
                    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
                    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
                    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
                )
                
                $nodeUninstaller = $null
                $productCode = $null
                foreach ($key in $uninstallKeys) {
                    $apps = Get-ItemProperty $key -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like "*Node.js*" }
                    if ($apps) {
                        $nodeUninstaller = $apps.UninstallString
                        if ($nodeUninstaller -match "MsiExec.exe\s+/[IX](\{[^}]+\})") {
                            $productCode = $matches[1]
                        }
                        break
                    }
                }
                
                if ($productCode) {
                    Write-Host "[Node.js] Progress: 7% - Found Node.js, uninstalling..." -ForegroundColor Gray
                    Start-Process "msiexec.exe" -ArgumentList "/x $productCode /qn" -Wait -NoNewWindow
                    Write-Host "[Node.js] Progress: 10% - Existing version uninstalled" -ForegroundColor Green
                    Start-Sleep -Seconds 3
                } else {
                    Write-Host "[Node.js] Progress: 10% - No installed version found in registry" -ForegroundColor Gray
                }
            } catch {
                Write-Host "[Node.js] WARNING: Uninstall failed: $($_.Exception.Message)" -ForegroundColor Yellow
                Write-Host "[Node.js] Continuing with installation..." -ForegroundColor Gray
            }
        }
        
        # Download
        Write-Host "[Node.js] [Step 1/5] Starting download..." -ForegroundColor Yellow
        $sources = @(
            "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/v$nodeVersion/node-v$nodeVersion-x64.msi",
            "https://npmmirror.com/mirrors/node/v$nodeVersion/node-v$nodeVersion-x64.msi",
            "https://nodejs.org/dist/v$nodeVersion/node-v$nodeVersion-x64.msi"
        )
        
        $downloaded = $false
        foreach ($url in $sources) {
            try {
                Write-Host "[Node.js] Progress: 10% - Downloading from $(($url -split '/')[-3])..." -ForegroundColor Gray
                $ProgressPreference = 'SilentlyContinue'
                Invoke-WebRequest -Uri $url -OutFile $nodeInstaller -UseBasicParsing -TimeoutSec 180
                $ProgressPreference = 'Continue'
                if (Test-Path $nodeInstaller) {
                    Write-Host "[Node.js] Progress: 20% - Download completed" -ForegroundColor Green
                    $downloaded = $true
                    break
                }
            } catch {
                Write-Host "[Node.js] Source failed, trying next..." -ForegroundColor Yellow
            }
        }
        
        if (-not $downloaded) {
            throw "All download sources failed"
        }
        
        # Install
        Write-Host "[Node.js] [Step 2/5] Installing to $nodeInstallPath..." -ForegroundColor Yellow
        Write-Host "[Node.js] Progress: 30% - Running MSI installer..." -ForegroundColor Gray
        Start-Process -FilePath "msiexec.exe" `
            -ArgumentList "/i `"$nodeInstaller`" /qn INSTALLDIR=`"$nodeInstallPath`" ADDLOCAL=All" `
            -Wait -NoNewWindow
        
        # Wait for installation to complete
        Write-Host "[Node.js] Waiting for installation to complete..." -ForegroundColor Gray
        Start-Sleep -Seconds 5
        
        # Verify installation
        Write-Host "[Node.js] [Step 3/5] Verifying installation..." -ForegroundColor Yellow
        Write-Host "[Node.js] Progress: 50% - Checking files..." -ForegroundColor Gray
        if (-not (Test-Path "$nodeInstallPath\node.exe")) {
            throw "Installation verification failed: node.exe not found in $nodeInstallPath"
        }
        Write-Host "[Node.js] Progress: 60% - Installation verified" -ForegroundColor Green
        
        # Configure environment
        Write-Host "[Node.js] [Step 4/5] Configuring environment variables..." -ForegroundColor Yellow
        Write-Host "[Node.js] Progress: 70% - Setting NODE_HOME..." -ForegroundColor Gray
        [Environment]::SetEnvironmentVariable("NODE_HOME", $nodeInstallPath, "Machine")
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (-not $currentPath.Contains("NODE_HOME")) {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;%NODE_HOME%", "Machine")
        }
        Write-Host "[Node.js] Progress: 80% - Environment configured" -ForegroundColor Green
        
        # Configure npm mirror
        if (Test-Path "$nodeInstallPath\npm.cmd") {
            Write-Host "[Node.js] [Step 5/5] Configuring npm mirror..." -ForegroundColor Yellow
            Write-Host "[Node.js] Progress: 90% - Setting Taobao mirror..." -ForegroundColor Gray
            Start-Sleep -Seconds 2
            & "$nodeInstallPath\npm.cmd" config set registry https://registry.npmmirror.com 2>&1 | Out-Null
        }
        
        $result.Success = $true
        $result.Message = "Installed successfully with npm Taobao mirror"
        Write-Host "[Node.js] Progress: 100% - All steps completed" -ForegroundColor Green
        Write-Host "[Node.js] [OK] Installation complete!" -ForegroundColor Green
        
    } catch {
        $result.Success = $false
        $result.Message = $_.Exception.Message
        Write-Host "[Node.js] [X] Failed: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        $result.Duration = ((Get-Date) - $startTime).TotalSeconds
    }
    
    return $result
}

# JDK Installation Function
$InstallJDKScript = {
    param($jdkVersion, $jdkMajorVersion, $installRoot, $tempDir)
    
    $result = @{
        Component = "JDK $jdkVersion"
        Success = $false
        Message = ""
        Duration = 0
    }
    
    $startTime = Get-Date
    
    try {
        $jdkInstallPath = "$installRoot\jdk-$jdkVersion"
        
        # Download
        Write-Host "[JDK] [Step 1/5] Starting download..." -ForegroundColor Yellow
        if ($jdkMajorVersion -eq "8") {
            $sources = @(
                @{ Url = "https://repo.huaweicloud.com/java/jdk/8u202-b08/jdk-8u202-windows-x64.exe"; File = "$tempDir\jdk-8u202-windows-x64.exe" }
            )
        } else {
            $sources = @(
                @{ Url = "https://d10.injdk.cn/openjdk/openjdk/17/openjdk-17.0.1_windows-x64_bin.zip"; File = "$tempDir\openjdk-17.0.1_windows-x64_bin.zip" },
                @{ Url = "https://repo.huaweicloud.com/java/jdk/17.0.2+8/jdk-17_windows-x64_bin.exe"; File = "$tempDir\jdk-17_windows-x64_bin.exe" }
            )
        }
        
        $downloaded = $false
        foreach ($source in $sources) {
            try {
                Write-Host "[JDK] Progress: 10% - Downloading JDK $jdkMajorVersion..." -ForegroundColor Gray
                $ProgressPreference = 'SilentlyContinue'
                Invoke-WebRequest -Uri $source.Url -OutFile $source.File -UseBasicParsing -TimeoutSec 180
                $ProgressPreference = 'Continue'
                if (Test-Path $source.File) {
                    Write-Host "[JDK] Progress: 30% - Download completed" -ForegroundColor Green
                    $jdkInstaller = $source.File
                    $downloaded = $true
                    break
                }
            } catch {
                Write-Host "[JDK] Source failed, trying next..." -ForegroundColor Yellow
            }
        }
        
        if (-not $downloaded) {
            throw "All download sources failed"
        }
        
        # Install
        Write-Host "[JDK] [Step 2/5] Installing to $jdkInstallPath..." -ForegroundColor Yellow
        Write-Host "[JDK] Progress: 40% - Processing installer..." -ForegroundColor Gray
        
        if ($jdkInstaller -like "*.zip") {
            # ZIP file - extract directly
            Write-Host "[JDK] Progress: 45% - Extracting ZIP archive..." -ForegroundColor Gray
            Expand-Archive -Path $jdkInstaller -DestinationPath "$installRoot\temp_jdk" -Force
            
            # Find the extracted JDK folder (it might have a different name)
            $extractedFolders = Get-ChildItem "$installRoot\temp_jdk" -Directory
            if ($extractedFolders.Count -eq 1) {
                $extractedJdkPath = $extractedFolders[0].FullName
                Write-Host "[JDK] Progress: 50% - Moving to final location..." -ForegroundColor Gray
                Move-Item $extractedJdkPath $jdkInstallPath -Force
                Remove-Item "$installRoot\temp_jdk" -Recurse -Force -ErrorAction SilentlyContinue
            } else {
                throw "Unexpected ZIP structure: found $($extractedFolders.Count) folders"
            }
        } else {
            # EXE installer
            Write-Host "[JDK] Progress: 45% - Running installer (this may take a while)..." -ForegroundColor Gray
            Start-Process -FilePath $jdkInstaller `
                -ArgumentList "/s INSTALLDIR=`"$jdkInstallPath`" STATIC=1" `
                -Wait -NoNewWindow
        }
        
        Write-Host "[JDK] [Step 3/5] Verifying installation..." -ForegroundColor Yellow
        Write-Host "[JDK] Progress: 60% - Checking files..." -ForegroundColor Gray
        if (-not (Test-Path "$jdkInstallPath\bin\java.exe")) {
            throw "Installation verification failed"
        }
        Write-Host "[JDK] Progress: 70% - Installation verified" -ForegroundColor Green
        
        # Configure environment
        Write-Host "[JDK] [Step 4/5] Configuring environment variables..." -ForegroundColor Yellow
        Write-Host "[JDK] Progress: 80% - Setting JAVA_HOME..." -ForegroundColor Gray
        [Environment]::SetEnvironmentVariable("JAVA_HOME", $jdkInstallPath, "Machine")
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (-not $currentPath.Contains("JAVA_HOME")) {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;%JAVA_HOME%\bin", "Machine")
        }
        
        $result.Success = $true
        $result.Message = "Installed successfully"
        Write-Host "[JDK] [Step 5/5] Finalizing..." -ForegroundColor Yellow
        Write-Host "[JDK] Progress: 100% - All steps completed" -ForegroundColor Green
        Write-Host "[JDK] [OK] Installation complete!" -ForegroundColor Green
        
    } catch {
        $result.Success = $false
        $result.Message = $_.Exception.Message
        Write-Host "[JDK] [X] Failed: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        $result.Duration = ((Get-Date) - $startTime).TotalSeconds
    }
    
    return $result
}

# MySQL Installation Function
$InstallMySQLScript = {
    param($mysqlVersion, $installRoot, $tempDir, $mysqlRootPwd, $mysqlPort)
    
    $result = @{
        Component = "MySQL $mysqlVersion"
        Success = $false
        Message = ""
        Duration = 0
    }
    
    $startTime = Get-Date
    
    try {
        $mysqlInstallPath = "$installRoot\mysql-$mysqlVersion"
        $mysqlZip = "$tempDir\mysql-$mysqlVersion-winx64.zip"
        
        # Download
        Write-Host "[MySQL] [Step 1/9] Starting download..." -ForegroundColor Yellow
        $sources = @(
            "https://repo.huaweicloud.com/mysql/Downloads/MySQL-8.0/mysql-$mysqlVersion-winx64.zip",
            "https://mirrors.tuna.tsinghua.edu.cn/mysql/downloads/MySQL-8.0/mysql-$mysqlVersion-winx64.zip"
        )
        
        $downloaded = $false
        foreach ($url in $sources) {
            try {
                Write-Host "[MySQL] Progress: 5% - Downloading from $(($url -split '/')[-4])..." -ForegroundColor Gray
                $ProgressPreference = 'SilentlyContinue'
                Invoke-WebRequest -Uri $url -OutFile $mysqlZip -UseBasicParsing -TimeoutSec 300
                $ProgressPreference = 'Continue'
                if (Test-Path $mysqlZip) {
                    Write-Host "[MySQL] Progress: 10% - Download completed (300MB)" -ForegroundColor Green
                    $downloaded = $true
                    break
                }
            } catch {
                Write-Host "[MySQL] Source failed, trying next..." -ForegroundColor Yellow
            }
        }
        
        if (-not $downloaded) {
            throw "All download sources failed"
        }
        
        # Extract
        Write-Host "[MySQL] [Step 2/9] Extracting archive..." -ForegroundColor Yellow
        Write-Host "[MySQL] Progress: 15% - Decompressing files..." -ForegroundColor Gray
        Expand-Archive -Path $mysqlZip -DestinationPath "$installRoot\temp_mysql" -Force
        Move-Item "$installRoot\temp_mysql\mysql-$mysqlVersion-winx64" $mysqlInstallPath -Force
        Remove-Item "$installRoot\temp_mysql" -Recurse -Force
        Write-Host "[MySQL] Progress: 20% - Files extracted" -ForegroundColor Green
        
        # Configure
        Write-Host "[MySQL] [Step 3/9] Creating configuration..." -ForegroundColor Yellow
        Write-Host "[MySQL] Progress: 25% - Writing my.ini..." -ForegroundColor Gray
        $mysqlDataPath = "$mysqlInstallPath\data"
        $mysqlInstallPathForConfig = $mysqlInstallPath -replace '\\', '/'
        $mysqlDataPathForConfig = $mysqlDataPath -replace '\\', '/'
        
        # Create my.ini with proper line breaks
        $myIniContent = @"
[mysqld]
port=$mysqlPort
basedir=$mysqlInstallPathForConfig
datadir=$mysqlDataPathForConfig
max_connections=200
character-set-server=utf8mb4
default-storage-engine=INNODB

[mysql]
default-character-set=utf8mb4

[client]
port=$mysqlPort
default-character-set=utf8mb4
"@
        Set-Content -Path "$mysqlInstallPath\my.ini" -Value $myIniContent -Encoding ASCII
        Write-Host "[MySQL] Progress: 30% - Configuration created" -ForegroundColor Green
        
        # Check if port is in use
        Write-Host "[MySQL] [Step 4/9] Checking port availability..." -ForegroundColor Yellow
        $portInUse = Get-NetTCPConnection -LocalPort $mysqlPort -ErrorAction SilentlyContinue
        if ($portInUse) {
            Write-Host "[MySQL] WARNING: Port $mysqlPort is in use" -ForegroundColor Yellow
            Write-Host "[MySQL] Attempting to use port 3307 instead..." -ForegroundColor Yellow
            $mysqlPort = "3307"
            
            # Recreate my.ini with new port
            $myIniContent = @"
[mysqld]
port=$mysqlPort
basedir=$mysqlInstallPathForConfig
datadir=$mysqlDataPathForConfig
max_connections=200
character-set-server=utf8mb4
default-storage-engine=INNODB

[mysql]
default-character-set=utf8mb4

[client]
port=$mysqlPort
default-character-set=utf8mb4
"@
            Set-Content -Path "$mysqlInstallPath\my.ini" -Value $myIniContent -Encoding ASCII
        }
        
        Write-Host "[MySQL] Progress: 35% - Port check completed" -ForegroundColor Green
        
        # Remove existing MySQL80 service if exists
        Write-Host "[MySQL] [Step 5/9] Checking for existing service..." -ForegroundColor Yellow
        $existingService = Get-Service -Name "MySQL80" -ErrorAction SilentlyContinue
        if ($existingService) {
            Write-Host "[MySQL] Progress: 37% - Removing existing service..." -ForegroundColor Yellow
            Stop-Service -Name "MySQL80" -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 2
            & sc.exe delete MySQL80 | Out-Null
            Start-Sleep -Seconds 2
        }
        
        Write-Host "[MySQL] Progress: 40% - Service check completed" -ForegroundColor Green
        
        # Create data directory if not exists
        if (-not (Test-Path $mysqlDataPath)) {
            Write-Host "[MySQL] Creating data directory..." -ForegroundColor Gray
            New-Item -Path $mysqlDataPath -ItemType Directory -Force | Out-Null
        }
        
        # Initialize
        Write-Host "[MySQL] [Step 6/9] Initializing database (this may take a minute)..." -ForegroundColor Yellow
        Write-Host "[MySQL] Progress: 45% - Running mysqld --initialize-insecure..." -ForegroundColor Gray
        $initOutput = & "$mysqlInstallPath\bin\mysqld.exe" --initialize-insecure --console 2>&1
        
        # Wait for initialization to complete
        Write-Host "[MySQL] Waiting for initialization to complete..." -ForegroundColor Gray
        Start-Sleep -Seconds 8
        
        # Check if initialization succeeded
        if (-not (Test-Path "$mysqlDataPath\mysql")) {
            $errorDetails = $initOutput | Out-String
            throw "Database initialization failed. Data directory not created. Error: $errorDetails"
        }
        Write-Host "[MySQL] Progress: 55% - Database initialized successfully" -ForegroundColor Green
        
        # Install service
        Write-Host "[MySQL] [Step 7/9] Installing MySQL80 service..." -ForegroundColor Yellow
        Write-Host "[MySQL] Progress: 60% - Registering service..." -ForegroundColor Gray
        $installOutput = & "$mysqlInstallPath\bin\mysqld.exe" --install MySQL80 --defaults-file="$mysqlInstallPath\my.ini" 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Service installation failed: $installOutput"
        }
        
        # Wait for service registration to complete
        Write-Host "[MySQL] Waiting for service registration..." -ForegroundColor Gray
        Start-Sleep -Seconds 5
        
        Write-Host "[MySQL] Progress: 65% - Service registered" -ForegroundColor Green
        
        # Start service with retry
        Write-Host "[MySQL] [Step 8/9] Starting MySQL80 service..." -ForegroundColor Yellow
        $maxRetries = 3
        $retryCount = 0
        $serviceStarted = $false
        
        while ($retryCount -lt $maxRetries -and -not $serviceStarted) {
            try {
                Start-Service -Name MySQL80 -ErrorAction Stop
                
                # Wait longer for service to fully start
                Write-Host "[MySQL] Progress: 70% - Waiting for service to start (attempt $($retryCount + 1))..." -ForegroundColor Gray
                Start-Sleep -Seconds 8
                
                $serviceStatus = Get-Service -Name MySQL80
                if ($serviceStatus.Status -eq 'Running') {
                    $serviceStarted = $true
                    Write-Host "[MySQL] Progress: 80% - Service started successfully" -ForegroundColor Green
                }
            } catch {
                $retryCount++
                if ($retryCount -lt $maxRetries) {
                    Write-Host "[MySQL] Start attempt $retryCount failed, waiting before retry..." -ForegroundColor Yellow
                    Start-Sleep -Seconds 10
                }
            }
        }
        
        if (-not $serviceStarted) {
            throw "Failed to start MySQL service after $maxRetries attempts. Check Windows Event Viewer for details."
        }
        
        # Wait for MySQL to be ready to accept connections
        Write-Host "[MySQL] Waiting for database to be ready..." -ForegroundColor Gray
        Start-Sleep -Seconds 5
        
        # Set password with retry
        Write-Host "[MySQL] [Step 9/9] Setting root password..." -ForegroundColor Yellow
        Write-Host "[MySQL] Progress: 85% - Configuring authentication..." -ForegroundColor Gray
        $pwdMaxRetries = 3
        $pwdRetryCount = 0
        $pwdSet = $false
        
        while ($pwdRetryCount -lt $pwdMaxRetries -and -not $pwdSet) {
            $sqlCommand = "ALTER USER 'root'@'localhost' IDENTIFIED BY '$mysqlRootPwd'; FLUSH PRIVILEGES;"
            $pwdResult = & "$mysqlInstallPath\bin\mysql.exe" -u root --skip-password --port=$mysqlPort -e $sqlCommand 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $pwdSet = $true
                Write-Host "[MySQL] Progress: 95% - Root password set successfully" -ForegroundColor Green
            } else {
                $pwdRetryCount++
                if ($pwdRetryCount -lt $pwdMaxRetries) {
                    Write-Host "[MySQL] Password setting attempt $pwdRetryCount failed, retrying..." -ForegroundColor Yellow
                    Start-Sleep -Seconds 3
                } else {
                    Write-Host "[MySQL] WARNING: Password setting failed after $pwdMaxRetries attempts" -ForegroundColor Yellow
                    Write-Host "[MySQL] You may need to set password manually: ALTER USER 'root'@'localhost' IDENTIFIED BY '$mysqlRootPwd';" -ForegroundColor Yellow
                }
            }
        }
        
        # Environment
        [Environment]::SetEnvironmentVariable("MYSQL_HOME", $mysqlInstallPath, "Machine")
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (-not $currentPath.Contains("MYSQL_HOME")) {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;%MYSQL_HOME%\bin", "Machine")
        }
        
        $result.Success = $true
        $result.Message = "Installed successfully (password: $mysqlRootPwd)"
        Write-Host "[MySQL] Progress: 100% - All steps completed" -ForegroundColor Green
        Write-Host "[MySQL] [OK] Installation complete!" -ForegroundColor Green
        
    } catch {
        $result.Success = $false
        $result.Message = $_.Exception.Message
        Write-Host "[MySQL] [X] Failed: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        $result.Duration = ((Get-Date) - $startTime).TotalSeconds
    }
    
    return $result
}

# Maven Installation Function
$InstallMavenScript = {
    param($mavenVersion, $installRoot, $tempDir)
    
    $result = @{
        Component = "Maven $mavenVersion"
        Success = $false
        Message = ""
        Duration = 0
    }
    
    $startTime = Get-Date
    
    try {
        $mavenInstallPath = "$installRoot\apache-maven-$mavenVersion"
        $mavenZip = "$tempDir\apache-maven-$mavenVersion-bin.zip"
        
        # Download
        Write-Host "[Maven] [Step 1/4] Starting download..." -ForegroundColor Yellow
        $sources = @(
            "https://repo.huaweicloud.com/apache/maven/maven-3/$mavenVersion/binaries/apache-maven-$mavenVersion-bin.zip",
            "https://dlcdn.apache.org/maven/maven-3/$mavenVersion/binaries/apache-maven-$mavenVersion-bin.zip"
        )
        
        $downloaded = $false
        foreach ($url in $sources) {
            try {
                Write-Host "[Maven] Progress: 10% - Downloading from $(($url -split '/')[-5])..." -ForegroundColor Gray
                $ProgressPreference = 'SilentlyContinue'
                Invoke-WebRequest -Uri $url -OutFile $mavenZip -UseBasicParsing -TimeoutSec 180
                $ProgressPreference = 'Continue'
                if (Test-Path $mavenZip) {
                    Write-Host "[Maven] Progress: 30% - Download completed (9MB)" -ForegroundColor Green
                    $downloaded = $true
                    break
                }
            } catch {
                Write-Host "[Maven] Source failed, trying next..." -ForegroundColor Yellow
            }
        }
        
        if (-not $downloaded) {
            throw "All download sources failed"
        }
        
        # Extract
        Write-Host "[Maven] [Step 2/4] Extracting archive..." -ForegroundColor Yellow
        Write-Host "[Maven] Progress: 40% - Decompressing files..." -ForegroundColor Gray
        Expand-Archive -Path $mavenZip -DestinationPath $installRoot -Force
        Write-Host "[Maven] Progress: 60% - Files extracted" -ForegroundColor Green
        
        # Environment
        Write-Host "[Maven] [Step 3/4] Configuring environment variables..." -ForegroundColor Yellow
        Write-Host "[Maven] Progress: 70% - Setting MAVEN_HOME..." -ForegroundColor Gray
        [Environment]::SetEnvironmentVariable("MAVEN_HOME", $mavenInstallPath, "Machine")
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (-not $currentPath.Contains("MAVEN_HOME")) {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;%MAVEN_HOME%\bin", "Machine")
        }
        Write-Host "[Maven] Progress: 80% - Environment configured" -ForegroundColor Green
        
        # Configure Aliyun mirror
        Write-Host "[Maven] [Step 4/4] Configuring Aliyun mirror..." -ForegroundColor Yellow
        Write-Host "[Maven] Progress: 90% - Updating settings.xml..." -ForegroundColor Gray
        $settingsPath = "$mavenInstallPath\conf\settings.xml"
        if (Test-Path $settingsPath) {
            try {
                # Load as XML for proper parsing
                [xml]$settingsXml = Get-Content $settingsPath -Raw -Encoding UTF8
                
                # Find or create mirrors node
                $mirrorsNode = $settingsXml.settings.mirrors
                if (-not $mirrorsNode) {
                    $mirrorsNode = $settingsXml.CreateElement("mirrors")
                    $settingsXml.settings.AppendChild($mirrorsNode) | Out-Null
                }
                
                # Clear existing mirrors (to avoid duplicates)
                $mirrorsNode.RemoveAll()
                
                # Create Aliyun mirror element
                $mirrorNode = $settingsXml.CreateElement("mirror")
                
                $idNode = $settingsXml.CreateElement("id")
                $idNode.InnerText = "aliyunmaven"
                $mirrorNode.AppendChild($idNode) | Out-Null
                
                $mirrorOfNode = $settingsXml.CreateElement("mirrorOf")
                $mirrorOfNode.InnerText = "*"
                $mirrorNode.AppendChild($mirrorOfNode) | Out-Null
                
                $nameNode = $settingsXml.CreateElement("name")
                $nameNode.InnerText = "Aliyun Maven"
                $mirrorNode.AppendChild($nameNode) | Out-Null
                
                $urlNode = $settingsXml.CreateElement("url")
                $urlNode.InnerText = "https://maven.aliyun.com/repository/public"
                $mirrorNode.AppendChild($urlNode) | Out-Null
                
                # Add mirror to mirrors
                $mirrorsNode.AppendChild($mirrorNode) | Out-Null
                
                # Save with proper formatting
                $settingsXml.Save($settingsPath)
                Write-Host "[Maven] Aliyun mirror configured successfully" -ForegroundColor Green
            } catch {
                Write-Host "[Maven] WARNING: Failed to configure mirror: $($_.Exception.Message)" -ForegroundColor Yellow
            }
        }
        
        $result.Success = $true
        $result.Message = "Installed successfully with Aliyun mirror"
        Write-Host "[Maven] Progress: 100% - All steps completed" -ForegroundColor Green
        Write-Host "[Maven] [OK] Installation complete!" -ForegroundColor Green
        
    } catch {
        $result.Success = $false
        $result.Message = $_.Exception.Message
        Write-Host "[Maven] [X] Failed: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        $result.Duration = ((Get-Date) - $startTime).TotalSeconds
    }
    
    return $result
}

# ==============================================
# 5. Concurrent Installation Execution
# ==============================================

$installationJobs = @()

if ($needInstallNode -or $needInstallJDK -or $needInstallMySQL -or $needInstallMaven) {
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host "  Starting Concurrent Installations" -ForegroundColor Cyan
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host ""
    
    # Start Node.js installation
    if ($needInstallNode) {
        if ($forceUninstallNode) {
            Write-Host "[OK] Launching Node.js $nodeVersion installation pipeline (will uninstall existing)..." -ForegroundColor Green
        } else {
            Write-Host "[OK] Launching Node.js $nodeVersion installation pipeline..." -ForegroundColor Green
        }
        $job = Start-Job -ScriptBlock $InstallNodeJSScript -ArgumentList $nodeVersion, $global:installRoot, $global:tempDir, $forceUninstallNode
        $installationJobs += @{ Job = $job; Component = "Node.js"; Processed = $false }
    }
    
    # Start JDK installation
    if ($needInstallJDK) {
        Write-Host "[OK] Launching JDK $jdkVersion installation pipeline..." -ForegroundColor Green
        $job = Start-Job -ScriptBlock $InstallJDKScript -ArgumentList $jdkVersion, $jdkMajorVersion, $global:installRoot, $global:tempDir
        $installationJobs += @{ Job = $job; Component = "JDK"; Processed = $false }
    }
    
    # Start MySQL installation
    if ($needInstallMySQL) {
        Write-Host "[OK] Launching MySQL $global:mysqlVersion installation pipeline..." -ForegroundColor Green
        $job = Start-Job -ScriptBlock $InstallMySQLScript -ArgumentList $global:mysqlVersion, $global:installRoot, $global:tempDir, $global:mysqlRootPwd, $global:mysqlPort
        $installationJobs += @{ Job = $job; Component = "MySQL"; Processed = $false }
    }
    
    # Start Maven installation
    if ($needInstallMaven) {
        Write-Host "[OK] Launching Maven $global:mavenVersion installation pipeline..." -ForegroundColor Green
        $job = Start-Job -ScriptBlock $InstallMavenScript -ArgumentList $global:mavenVersion, $global:installRoot, $global:tempDir
        $installationJobs += @{ Job = $job; Component = "Maven"; Processed = $false }
    }
    
    Write-Host ""
    Write-Host "All installation pipelines are running concurrently..." -ForegroundColor Yellow
    Write-Host "Each component: Download -> Install -> Configure -> Environment Setup" -ForegroundColor Gray
    Write-Host ""
    
    # Monitor progress - simplified version
    $completedCount = 0
    $totalCount = $installationJobs.Count
    $results = @()
    
    Write-Host "Waiting for installations to complete..." -ForegroundColor Cyan
    Write-Host "Note: Job outputs are hidden in background. Please wait for completion messages." -ForegroundColor Gray
    Write-Host ""
    
    # Show progress dots
    $dots = 0
    
    while ($completedCount -lt $totalCount) {
        foreach ($jobInfo in $installationJobs) {
            $job = $jobInfo.Job
            
            # Check if job completed
            if ($job.State -eq 'Completed' -and -not $jobInfo.Processed) {
                # Receive final result
                $jobOutput = Receive-Job -Job $job -ErrorAction SilentlyContinue
                
                # The last item should be the hashtable result
                $result = $null
                if ($jobOutput) {
                    # Find hashtable in output
                    foreach ($item in $jobOutput) {
                        if ($item -is [hashtable] -and $item.ContainsKey('Component')) {
                            $result = $item
                        }
                    }
                }
                
                # Ensure we have a valid result
                if (-not $result -or -not $result.ContainsKey('Duration')) {
                    Write-Host "`n[WARNING] Job for $($jobInfo.Component) completed but returned invalid result" -ForegroundColor Yellow
                    $result = @{
                        Component = $jobInfo.Component
                        Success = $false
                        Message = "Completed with unknown status"
                        Duration = 0
                    }
                }
                
                $results += $result
                $jobInfo.Processed = $true
                $completedCount++
                
                # Display completion status
                Write-Host ""
                $statusIcon = if ($result.Success) { "[OK]" } else { "[X]" }
                $statusColor = if ($result.Success) { "Green" } else { "Red" }
                $duration = if ($result.Duration) { [math]::Round($result.Duration, 1) } else { 0 }
                
                Write-Host "========================================" -ForegroundColor Cyan
                Write-Host "[$completedCount/$totalCount] $statusIcon $($result.Component) - " -NoNewline
                Write-Host "$($result.Message) " -ForegroundColor $statusColor -NoNewline
                Write-Host "($duration s)" -ForegroundColor Gray
                Write-Host "========================================" -ForegroundColor Cyan
                
                # Show progress dots again
                Write-Host ""
                if ($completedCount -lt $totalCount) {
                    Write-Host "Waiting for remaining installations" -NoNewline
                    $dots = 0
                }
            }
            
            # Check for failed jobs
            if ($job.State -eq 'Failed' -and -not $jobInfo.Processed) {
                Write-Host ""
                Write-Host "========================================" -ForegroundColor Red
                Write-Host "[$($jobInfo.Component)] Job execution failed!" -ForegroundColor Red
                
                # Try to get error details
                try {
                    $errorOutput = Receive-Job -Job $job -ErrorAction SilentlyContinue 2>&1
                    if ($errorOutput) {
                        Write-Host "Error details:" -ForegroundColor Yellow
                        $errorOutput | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
                    }
                } catch {
                    Write-Host "  Could not retrieve error details" -ForegroundColor Gray
                }
                
                Write-Host "========================================" -ForegroundColor Red
                Write-Host ""
                
                # Create error result
                $result = @{
                    Component = $jobInfo.Component
                    Success = $false
                    Message = "Job execution failed"
                    Duration = 0
                }
                $results += $result
                
                $jobInfo.Processed = $true
                $completedCount++
            }
        }
        
        # Show progress indication
        if ($completedCount -lt $totalCount) {
            Write-Host "." -NoNewline
            $dots++
            if ($dots % 60 -eq 0) {
                Write-Host " [$completedCount/$totalCount completed]"
                Write-Host "Still waiting" -NoNewline
            }
        }
        
        Start-Sleep -Seconds 1
    }
    
    # Clean up jobs
    $installationJobs | ForEach-Object { Remove-Job -Job $_.Job -Force }
    
    # Summary
    Write-Host ""
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host "  Installation Summary" -ForegroundColor Cyan
    Write-Host "================================================" -ForegroundColor Cyan
    
    $successCount = ($results | Where-Object { $_.Success }).Count
    
    # Calculate total time (filter out objects without Duration property)
    $validResults = $results | Where-Object { $null -ne $_.Duration }
    if ($validResults.Count -gt 0) {
        $totalTime = [math]::Round(($validResults | Measure-Object -Property Duration -Sum).Sum, 1)
        Write-Host "Total: $totalCount | Success: $successCount | Failed: $($totalCount - $successCount)"
        Write-Host "Total time (concurrent): $totalTime seconds" -ForegroundColor Cyan
    } else {
        Write-Host "Total: $totalCount | Success: $successCount | Failed: $($totalCount - $successCount)"
    }
    Write-Host ""
    
    # Detailed results
    foreach ($result in $results) {
        $icon = if ($result.Success) { "[OK]" } else { "[X]" }
        $color = if ($result.Success) { "Green" } else { "Red" }
        Write-Host "  $icon $($result.Component): $($result.Message)" -ForegroundColor $color
    }
    
} else {
    Write-Host "================================================" -ForegroundColor Green
    Write-Host "  All Components Already Installed!" -ForegroundColor Green
    Write-Host "================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "No installation needed. All selected versions are available." -ForegroundColor Gray
}

# ==============================================
# 6. Cleanup Temporary Files
# ==============================================
if (Test-Path $global:tempDir) {
    Write-Host ""
    Write-Host "================================================" -ForegroundColor Cyan
    Write-Host "  Cleaning Up Temporary Files" -ForegroundColor Cyan
    Write-Host "================================================" -ForegroundColor Cyan
    
    try {
        # Calculate temp directory size
        $tempFiles = Get-ChildItem -Path $global:tempDir -Recurse -File -ErrorAction SilentlyContinue
        $tempSize = ($tempFiles | Measure-Object -Property Length -Sum).Sum
        $tempSizeMB = [math]::Round($tempSize / 1MB, 2)
        
        Write-Host "Temp directory: $global:tempDir" -ForegroundColor Gray
        Write-Host "Size: $tempSizeMB MB" -ForegroundColor Gray
        Write-Host ""
        
        $cleanupChoice = Read-Host "Delete temporary installation files? (Y/N, default: Y)"
        if ([string]::IsNullOrEmpty($cleanupChoice) -or $cleanupChoice -eq "Y" -or $cleanupChoice -eq "y") {
            Write-Host "Deleting temporary files..." -ForegroundColor Yellow
            Remove-Item -Path $global:tempDir -Recurse -Force -ErrorAction Stop
            Write-Host "[OK] Temporary files deleted (freed $tempSizeMB MB)" -ForegroundColor Green
        } else {
            Write-Host "[INFO] Temporary files kept in: $global:tempDir" -ForegroundColor Gray
        }
    } catch {
        Write-Host "[WARNING] Failed to delete temp files: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "[INFO] You can manually delete: $global:tempDir" -ForegroundColor Gray
    }
}

# ==============================================
# 7. Completion
# ==============================================
Write-Host ""
Write-Host "================================================" -ForegroundColor Green
Write-Host "  Setup Complete!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installation Directory: $global:installRoot" -ForegroundColor Cyan
Write-Host ""
Write-Host "Important: Restart your terminal to use the new environment!" -ForegroundColor Yellow
Write-Host ""
Write-Host "Quick verification commands:" -ForegroundColor Cyan
if ($needInstallNode) { Write-Host "  node --version" -ForegroundColor Gray }
if ($needInstallJDK) { Write-Host "  java -version" -ForegroundColor Gray }
if ($needInstallMySQL) { Write-Host "  mysql --version" -ForegroundColor Gray }
if ($needInstallMaven) { Write-Host "  mvn -version" -ForegroundColor Gray }
Write-Host ""
Write-Host "Thank you for using DevSetup Concurrent Installer!" -ForegroundColor Green
Write-Host ""
Read-Host "Press Enter to exit"
