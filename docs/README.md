<!-- Custom Table Styling for local markdown previewers (VS Code, Obsidian, etc.) to match the Mermaid chart's purple theme -->
<style>
  table {
    border-collapse: collapse;
    width: 100%;
    margin: 24px 0;
    color: #e8e0f0 !important;
    background-color: #1a1020 !important;
    border: 1px solid #ba53e6 !important;
  }
  th {
    background-color: #1f1630 !important;
    color: #e8e0f0 !important;
    border: 1px solid #ba53e6 !important;
    padding: 12px;
    font-weight: bold;
  }
  td {
    border: 1px solid #ba53e6 !important;
    padding: 12px;
  }
  tr:nth-child(even) {
    background-color: #1f1630 !important;
  }
  /* Style inline code blocks inside tables to match */
  table code {
    background-color: #2a1635 !important;
    color: #e8d0ff !important;
    border: 1px solid #ba53e6 !important;
    padding: 2px 6px !important;
  }
</style>

# TypeSymbol documentation

| Document | Description |
| --- | --- |
| [install.md](install.md) | Official macOS Homebrew install path |
| [releasing.md](releasing.md) | Tags, GitHub releases, package manager automation |
| [homebrew-tap.md](homebrew-tap.md) | Custom Homebrew tap and CI |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Building, testing, fork packaging |
| [SECURITY.md](SECURITY.md) | Responsible disclosure |
| [syntax-guide.md](syntax-guide.md) | Shorthand syntax reference |
