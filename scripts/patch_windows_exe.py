import os
import sys
import struct
import shutil
import zipfile
import ctypes
from ctypes import wintypes

def patch_exe():
    kernel32 = ctypes.windll.kernel32

    BeginUpdateResource = kernel32.BeginUpdateResourceW
    BeginUpdateResource.restype = wintypes.HANDLE
    BeginUpdateResource.argtypes = [wintypes.LPCWSTR, wintypes.BOOL]

    UpdateResource = kernel32.UpdateResourceW
    UpdateResource.restype = wintypes.BOOL
    UpdateResource.argtypes = [
        wintypes.HANDLE,
        ctypes.c_void_p,
        ctypes.c_void_p,
        wintypes.WORD,
        ctypes.c_char_p,
        wintypes.DWORD
    ]

    EndUpdateResource = kernel32.EndUpdateResourceW
    EndUpdateResource.restype = wintypes.BOOL
    EndUpdateResource.argtypes = [wintypes.HANDLE, wintypes.BOOL]

    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target_candidates = [
        os.path.join(project_root, "src-tauri", "target", "x86_64-pc-windows-gnu", "release", "scrdeck.exe"),
        os.path.join(project_root, "src-tauri", "target", "release", "scrdeck.exe"),
        os.path.join(project_root, "release", "scrdeck.exe"),
    ]

    found_any = False
    for exe_path in target_candidates:
        if not os.path.exists(exe_path):
            continue
        found_any = True
        print(f"[patch_windows_exe] Patching resources for: {exe_path}")

        manifest_xml = (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'
            '<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">\n'
            '  <dependency>\n'
            '    <dependentAssembly>\n'
            '      <assemblyIdentity\n'
            '        type="win32"\n'
            '        name="Microsoft.Windows.Common-Controls"\n'
            '        version="6.0.0.0"\n'
            '        processorArchitecture="*"\n'
            '        publicKeyToken="6595b64144ccf1df"\n'
            '        language="*"\n'
            '      />\n'
            '    </dependentAssembly>\n'
            '  </dependency>\n'
            '</assembly>'
        ).encode("utf-8")

        # Also save external manifest as backup in target directory
        if "target" in exe_path:
            manifest_backup_path = exe_path + ".manifest"
            try:
                with open(manifest_backup_path, "wb") as f:
                    f.write(manifest_xml)
            except Exception as e:
                print(f"[patch_windows_exe] Warning: Could not write external manifest: {e}")

        h = BeginUpdateResource(exe_path, False)
        if not h:
            err = ctypes.get_last_error()
            print(f"[patch_windows_exe] Warning: BeginUpdateResource failed for {exe_path} with error code {err}")
            continue

        try:
            # RT_MANIFEST = 24, CREATEPROCESS_MANIFEST_RESOURCE_ID = 1
            RT_MANIFEST = 24
            MANIFEST_ID = 1
            if not UpdateResource(h, RT_MANIFEST, MANIFEST_ID, 0, manifest_xml, len(manifest_xml)):
                raise ctypes.WinError(ctypes.get_last_error())

            # Embed icons if icon.ico exists
            icon_path = os.path.join(project_root, "src-tauri", "icons", "icon.ico")
            if os.path.exists(icon_path):
                with open(icon_path, "rb") as f:
                    ico_data = f.read()

                reserved, itype, count = struct.unpack("<HHH", ico_data[:6])
                if itype == 1 and count > 0:
                    RT_ICON = 3
                    RT_GROUP_ICON = 14
                    grp_data = bytearray(ico_data[:6])
                    cursor = 6

                    for i in range(count):
                        entry = ico_data[cursor:cursor+16]
                        w, h_val, colors, res, planes, bpp, size, img_offset = struct.unpack("<BBBBHHII", entry)
                        img_data = ico_data[img_offset:img_offset+size]
                        icon_id = i + 1

                        UpdateResource(h, RT_ICON, icon_id, 0, img_data, len(img_data))
                        grp_data.extend(entry[:12])
                        grp_data.extend(struct.pack("<H", icon_id))
                        cursor += 16

                    grp_bytes = bytes(grp_data)
                    UpdateResource(h, RT_GROUP_ICON, 32512, 0, grp_bytes, len(grp_bytes))
                    UpdateResource(h, RT_GROUP_ICON, 1, 0, grp_bytes, len(grp_bytes))

            if not EndUpdateResource(h, False):
                raise ctypes.WinError(ctypes.get_last_error())

            print(f"[patch_windows_exe] Successfully embedded Common-Controls manifest and application icons into {os.path.basename(exe_path)}!")
        except Exception as e:
            EndUpdateResource(h, True)
            print(f"[patch_windows_exe] Error while updating resources for {exe_path}: {e}")

    # Synchronize WebView2Loader.dll and package release assets
    sync_release_artifacts(project_root)

    if not found_any:
        print("[patch_windows_exe] Notice: scrdeck.exe not found in candidates, skipping.")
    return

def sync_release_artifacts(project_root):
    release_dir = os.path.join(project_root, "release")
    os.makedirs(release_dir, exist_ok=True)

    target_gnu_release = os.path.join(project_root, "src-tauri", "target", "x86_64-pc-windows-gnu", "release")
    src_tauri_bin = os.path.join(project_root, "src-tauri", "bin")
    os.makedirs(src_tauri_bin, exist_ok=True)

    # Locate WebView2Loader.dll
    dll_candidates = [
        os.path.join(target_gnu_release, "WebView2Loader.dll"),
        os.path.join(src_tauri_bin, "WebView2Loader.dll"),
        os.path.join(release_dir, "WebView2Loader.dll")
    ]
    found_dll = None
    for cand in dll_candidates:
        if os.path.exists(cand):
            found_dll = cand
            break

    if found_dll:
        # Sync to src-tauri/bin and release
        bin_dll = os.path.join(src_tauri_bin, "WebView2Loader.dll")
        rel_dll = os.path.join(release_dir, "WebView2Loader.dll")
        if found_dll != bin_dll:
            shutil.copy2(found_dll, bin_dll)
        if found_dll != rel_dll:
            shutil.copy2(found_dll, rel_dll)
        print(f"[patch_windows_exe] Synced WebView2Loader.dll to {bin_dll} and {rel_dll}")

    # Sync scrdeck.exe to release/
    target_exe = os.path.join(target_gnu_release, "scrdeck.exe")
    rel_exe = os.path.join(release_dir, "scrdeck.exe")
    if os.path.exists(target_exe):
        shutil.copy2(target_exe, rel_exe)
        print(f"[patch_windows_exe] Synced scrdeck.exe to {rel_exe}")

    # Sync NSIS setup if present
    nsis_exe = os.path.join(target_gnu_release, "bundle", "nsis", "ScrDeck_0.1.0_x64-setup.exe")
    rel_setup = os.path.join(release_dir, "ScrDeck-v1.0.0-Setup.exe")
    if os.path.exists(nsis_exe):
        shutil.copy2(nsis_exe, rel_setup)
        print(f"[patch_windows_exe] Synced NSIS installer to {rel_setup}")

    # Build Portable ZIP if both scrdeck.exe and WebView2Loader.dll exist
    if os.path.exists(rel_exe) and os.path.exists(os.path.join(release_dir, "WebView2Loader.dll")):
        zip_path = os.path.join(release_dir, "ScrDeck-v1.0.0-Portable.zip")
        readme_content = (
            "ScrDeck Portable v1.0.0\n"
            "=======================\n\n"
            "Quick Start:\n"
            "1. Extract all files to a folder of your choice.\n"
            "2. Double-click `scrdeck.exe` to launch ScrDeck.\n"
            "3. Ensure `WebView2Loader.dll` remains in the same directory as `scrdeck.exe`.\n\n"
            "Requirements:\n"
            "- Windows 10 (Build 1809+) or Windows 11\n"
            "- Microsoft Edge WebView2 Runtime (preinstalled on Windows 10/11, or auto-installed)\n\n"
            "Documentation & Support:\n"
            "https://github.com/ronnanakibu/scrdeck\n"
        )
        with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
            z.write(rel_exe, "ScrDeck-Portable/scrdeck.exe")
            z.write(os.path.join(release_dir, "WebView2Loader.dll"), "ScrDeck-Portable/WebView2Loader.dll")
            z.writestr("ScrDeck-Portable/README.txt", readme_content)
        print(f"[patch_windows_exe] Packaged portable archive: {zip_path}")

if __name__ == "__main__":
    patch_exe()
