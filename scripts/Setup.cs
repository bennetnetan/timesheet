using System;
using System.Drawing;
using System.IO;
using System.IO.Compression;
using System.Reflection;
using System.Diagnostics;
using System.Windows.Forms;
using Microsoft.Win32;

namespace TimesheetInstaller
{
    public class SetupForm : Form
    {
        private TextBox txtPath;
        private CheckBox chkDesktop;
        private CheckBox chkStartMenu;
        private CheckBox chkLaunch;
        private Button btnInstall;
        private Button btnBrowse;
        private Label lblStatus;
        private ProgressBar prgBar;

        public SetupForm()
        {
            this.Text = "Timesheet Setup (v0.11.1)";
            this.Size = new Size(520, 390);
            this.StartPosition = FormStartPosition.CenterScreen;
            this.FormBorderStyle = FormBorderStyle.FixedDialog;
            this.MaximizeBox = false;
            this.Font = new Font("Segoe UI", 9.5f);
            this.BackColor = Color.FromArgb(248, 250, 252);

            // Header Banner
            Panel header = new Panel {
                Dock = DockStyle.Top,
                Height = 85,
                BackColor = Color.FromArgb(30, 41, 59)
            };
            Label lblTitle = new Label {
                Text = "⏱ Timesheet Desktop",
                Font = new Font("Segoe UI Semibold", 16f, FontStyle.Bold),
                ForeColor = Color.White,
                Location = new Point(24, 16),
                AutoSize = true
            };
            Label lblSubtitle = new Label {
                Text = "Offline-first Windows desktop timesheet application",
                Font = new Font("Segoe UI", 9.5f),
                ForeColor = Color.FromArgb(148, 163, 184),
                Location = new Point(26, 48),
                AutoSize = true
            };
            header.Controls.Add(lblTitle);
            header.Controls.Add(lblSubtitle);
            this.Controls.Add(header);

            // Install Directory Label & Input
            Label lblDir = new Label {
                Text = "Destination Folder:",
                Location = new Point(24, 105),
                AutoSize = true,
                Font = new Font("Segoe UI Semibold", 9.5f)
            };
            this.Controls.Add(lblDir);

            string defaultPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "Timesheet");
            txtPath = new TextBox {
                Text = defaultPath,
                Location = new Point(24, 130),
                Width = 360
            };
            this.Controls.Add(txtPath);

            btnBrowse = new Button {
                Text = "Browse...",
                Location = new Point(395, 128),
                Width = 85,
                Height = 28
            };
            btnBrowse.Click += (s, e) => {
                using (var fbd = new FolderBrowserDialog()) {
                    fbd.SelectedPath = txtPath.Text;
                    if (fbd.ShowDialog() == DialogResult.OK) {
                        txtPath.Text = fbd.SelectedPath;
                    }
                }
            };
            this.Controls.Add(btnBrowse);

            // Options
            chkDesktop = new CheckBox {
                Text = "Create Desktop shortcut",
                Checked = true,
                Location = new Point(26, 170),
                AutoSize = true
            };
            chkStartMenu = new CheckBox {
                Text = "Create Start Menu shortcut",
                Checked = true,
                Location = new Point(26, 200),
                AutoSize = true
            };
            chkLaunch = new CheckBox {
                Text = "Launch Timesheet after installation",
                Checked = true,
                Location = new Point(26, 230),
                AutoSize = true
            };
            this.Controls.Add(chkDesktop);
            this.Controls.Add(chkStartMenu);
            this.Controls.Add(chkLaunch);

            // Progress Bar & Status
            prgBar = new ProgressBar {
                Location = new Point(24, 268),
                Width = 456,
                Height = 16,
                Visible = false
            };
            this.Controls.Add(prgBar);

            lblStatus = new Label {
                Location = new Point(24, 295),
                Width = 320,
                Height = 36,
                ForeColor = Color.FromArgb(71, 85, 105),
                Text = "Ready to install."
            };
            this.Controls.Add(lblStatus);

            // Install Button
            btnInstall = new Button {
                Text = "Install Timesheet",
                Font = new Font("Segoe UI Semibold", 10f, FontStyle.Bold),
                Location = new Point(345, 290),
                Width = 135,
                Height = 36,
                BackColor = Color.FromArgb(37, 99, 235),
                ForeColor = Color.White,
                FlatStyle = FlatStyle.Flat
            };
            btnInstall.FlatAppearance.BorderSize = 0;
            btnInstall.Click += BtnInstall_Click;
            this.Controls.Add(btnInstall);
        }

        private void BtnInstall_Click(object sender, EventArgs e)
        {
            try
            {
                string targetDir = txtPath.Text.Trim();
                if (string.IsNullOrEmpty(targetDir)) return;

                btnInstall.Enabled = false;
                btnBrowse.Enabled = false;
                txtPath.Enabled = false;
                chkDesktop.Enabled = false;
                chkStartMenu.Enabled = false;
                chkLaunch.Enabled = false;

                prgBar.Visible = true;
                prgBar.Style = ProgressBarStyle.Marquee;
                lblStatus.Text = "Extracting files to " + targetDir + "...";
                Application.DoEvents();

                // 1. Kill running instances
                foreach (var p in Process.GetProcessesByName("timesheet")) {
                    try { p.Kill(); p.WaitForExit(2000); } catch {}
                }

                // 2. Extract embedded package
                if (!Directory.Exists(targetDir)) {
                    Directory.CreateDirectory(targetDir);
                }

                Assembly asm = Assembly.GetExecutingAssembly();
                using (Stream stream = asm.GetManifestResourceStream("Package.zip")) {
                    if (stream == null) throw new Exception("Embedded installation package 'Package.zip' not found.");
                    using (ZipArchive archive = new ZipArchive(stream)) {
                        foreach (var entry in archive.Entries) {
                            if (string.IsNullOrEmpty(entry.Name)) continue; // skip dirs
                            string destFile = Path.Combine(targetDir, entry.FullName);
                            string destSubDir = Path.GetDirectoryName(destFile);
                            if (!Directory.Exists(destSubDir)) Directory.CreateDirectory(destSubDir);
                            entry.ExtractToFile(destFile, true);
                        }
                    }
                }

                lblStatus.Text = "Creating shortcuts...";
                Application.DoEvents();

                string exePath = Path.Combine(targetDir, "timesheet.exe");

                // 3. Create Start Menu shortcut
                if (chkStartMenu.Checked) {
                    string startDir = Environment.GetFolderPath(Environment.SpecialFolder.Programs);
                    CreateShortcut(Path.Combine(startDir, "Timesheet.lnk"), exePath, targetDir);
                }

                // 4. Create Desktop shortcut
                if (chkDesktop.Checked) {
                    string desktopDir = Environment.GetFolderPath(Environment.SpecialFolder.Desktop);
                    CreateShortcut(Path.Combine(desktopDir, "Timesheet.lnk"), exePath, targetDir);
                }

                // 5. Register in Windows Uninstall Registry
                lblStatus.Text = "Registering Windows application...";
                Application.DoEvents();

                using (RegistryKey key = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\Timesheet")) {
                    if (key != null) {
                        key.SetValue("DisplayName", "Timesheet");
                        key.SetValue("DisplayVersion", "0.11.1");
                        key.SetValue("Publisher", "Timesheet Desktop");
                        key.SetValue("DisplayIcon", exePath + ",0");
                        key.SetValue("InstallLocation", targetDir);
                        string uninstBat = Path.Combine(targetDir, "uninstall.bat");
                        key.SetValue("UninstallString", "\"" + uninstBat + "\"");
                    }
                }

                prgBar.Style = ProgressBarStyle.Blocks;
                prgBar.Value = 100;
                lblStatus.Text = "✓ Installation completed successfully!";
                lblStatus.ForeColor = Color.FromArgb(16, 185, 129);

                btnInstall.Text = "Finished";
                btnInstall.Enabled = true;
                btnInstall.Click -= BtnInstall_Click;
                btnInstall.Click += (s, ev) => this.Close();

                if (chkLaunch.Checked && File.Exists(exePath)) {
                    Process.Start(new ProcessStartInfo(exePath) { WorkingDirectory = targetDir });
                    this.Close();
                }
            }
            catch (Exception ex)
            {
                prgBar.Visible = false;
                lblStatus.ForeColor = Color.Red;
                lblStatus.Text = "Error: " + ex.Message;
                MessageBox.Show("Installation failed:\n" + ex.Message, "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                btnInstall.Enabled = true;
            }
        }

        private static void CreateShortcut(string shortcutPath, string targetPath, string workingDir)
        {
            Type t = Type.GetTypeFromProgID("WScript.Shell");
            dynamic shell = Activator.CreateInstance(t);
            dynamic shortcut = shell.CreateShortcut(shortcutPath);
            shortcut.TargetPath = targetPath;
            shortcut.WorkingDirectory = workingDir;
            shortcut.Description = "Timesheet - Offline-first Windows Desktop Timesheet";
            shortcut.Save();
        }

        [STAThread]
        static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new SetupForm());
        }
    }
}
