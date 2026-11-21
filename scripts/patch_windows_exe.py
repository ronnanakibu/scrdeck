import os
import sys
import struct
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

    if not found_any:
        print("[patch_windows_exe] Notice: scrdeck.exe not found in candidates, skipping.")
    return

if __name__ == "__main__":
    patch_exe()
