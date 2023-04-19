import urllib.request, logging, subprocess, os, shutil

logger = logging.getLogger()
logging.basicConfig(format='%(asctime)s [%(levelname)s]: %(message)s', level=logging.INFO)

# NOTE:
# - This script is to let you export template overrides, to put in manual bug fixes to Open API templating
# - This script outputs to the git-ignored `template-overrides-source` folder.
#   Be sure to copy over the relevant buggy files to the `template-overrides` folder, source control them,
#   and then amend them there in a following commit.
#   Be sure to mark the lines you've changed by adding a comment {{! RADIX CHANGE ON NEXT LINE }} on the
#   previous line to any changes.
#- Running the script assumes you've previously run `regenerate.py` to download the open api generator

OPENAPI_GENERATION_FOLDER='.'
OPENAPI_GENERATOR_FIXED_VERSION_JAR=os.path.join(OPENAPI_GENERATION_FOLDER, 'openapi-generator-cli-6.2.1.jar')

def run(command, cwd = '.', should_log = False):
    if (should_log): logging.debug('Running cmd: %s' % command)
    response = subprocess.run(' '.join(command), cwd=cwd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stderr = response.stderr.decode('utf-8')
    if response.returncode != 0: raise Exception(stderr)
    stdout = response.stdout.decode('utf-8')
    if (should_log): logging.debug('Response: %s', stdout)
    return stdout

if __name__ == "__main__":
    # Set working directory to be the same directory as this script
    os.chdir(os.path.dirname(os.path.abspath(__file__)))

    run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR,
         'author', 'template',
         '-g', 'typescript-fetch',
         '-o', 'template-overrides-source',
         ], should_log=True)