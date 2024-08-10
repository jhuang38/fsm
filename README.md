# fsm
This is a project to automatically manage your file system. More documentation is TBA:
- Add more possible data sources (e.g. taking from discord servers)
- Improve error handling
- Include better documentation on how to configure behavior (i.e. via config file and filters)
- Add proper test cases

The functionality of this utility is based around a config file. See `fsm_config_example.json` as an example, but the basic functionality involves choosing a path to watch, a path to build a managed directory from, and various filters to organize files based on filename (regex), extension, etc.