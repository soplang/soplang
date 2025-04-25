# -*- mode: python ; coding: utf-8 -*-

block_cipher = None

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[('src', 'src')],
    hiddenimports=[
        'src.core',
        'src.runtime',
        'src.stdlib',
        'src.utils',
        'colorama',
        'prompt_toolkit',
        'prompt_toolkit.clipboard',
        'prompt_toolkit.completion',
        'prompt_toolkit.filters',
        'prompt_toolkit.history',
        'prompt_toolkit.key_binding',
        'prompt_toolkit.layout',
        'prompt_toolkit.lexers',
        'prompt_toolkit.styles',
        'prompt_toolkit.shortcuts',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='soplang',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon='windows/soplang_icon.ico',
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='soplang',
)
