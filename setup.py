from setuptools import setup, find_packages

setup(
    name="gcm",
    version="0.1.0",
    description="Generate intelligent git commit messages using LLMs",
    author="Your Name",
    packages=find_packages(),
    install_requires=[
        "openai>=1.0.0",
        "GitPython>=3.1.0",
        "PyYAML>=6.0",
        "click>=8.0.0",
    ],
    entry_points={
        "console_scripts": [
            "gcm=gcm.cli:main",
        ],
    },
    python_requires=">=3.8",
)