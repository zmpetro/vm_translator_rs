import subprocess
import sys

subprocess.call(['./vm_translator_rs', sys.argv[1]])
