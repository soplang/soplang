#!/usr/bin/env python3
from setuptools import find_packages, setup

# Read requirements from requirements.txt
with open("requirements.txt") as f:
    requirements = [
        line.strip() for line in f if not line.startswith("#") and line.strip()
    ]

# Windows-specific requirements
windows_requirements = [
    "pypiwin32",
    "pywin32",
]

setup(
    name="soplang",
    version="0.1.0",
    description="The Somali Programming Language",
    author="Sharafdin",
    author_email="info@soplang.org",
    url="https://www.soplang.org/",
    packages=find_packages(),
    include_package_data=True,
    install_requires=requirements,
    extras_require={
        "windows": windows_requirements,
    },
    entry_points={
        "console_scripts": [
            "soplang=main:main",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Topic :: Software Development :: Interpreters",
    ],
    python_requires=">=3.6",
)
