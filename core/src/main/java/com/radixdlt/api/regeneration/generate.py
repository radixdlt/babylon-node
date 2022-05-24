import urllib.request, logging, subprocess, os, shutil

# Inspired by https://github.com/radixdlt/radixdlt-python-clients
# Requires python 3+ and various packages above

logger = logging.getLogger()
logging.basicConfig(format='%(asctime)s [%(levelname)s]: %(message)s', level=logging.INFO)

CORE_API_SPEC_LOCATION = '../core/core-api-spec.yaml'
CORE_API_GENERATED_DESTINATION = '../core/generated/'

COMMON_API_PACKAGE = 'com.radixdlt.api.common'
CORE_API_PACKAGE = 'com.radixdlt.api.core'

OPENAPI_GENERATION_FOLDER='.'
OPENAPI_TEMP_GENERATION_FOLDER='./temp'
OPENAPI_GENERATOR_FIXED_VERSION_JAR=os.path.join(OPENAPI_GENERATION_FOLDER, 'openapi-generator-cli-5.2.1.jar')
OPENAPI_GENERATOR_FIXED_VERSION_DOWNLOAD_URL='https://search.maven.org/remotecontent?filepath=org/openapitools/openapi-generator-cli/5.2.1/openapi-generator-cli-5.2.1.jar'

def safe_os_remove(path, silent = False):
    try:
        shutil.rmtree(path) if os.path.isdir(path) else os.remove(path)
    except Exception as e:
        if not silent: logger.warning(e)
        
def replace_in_file(filename, target, replacement):
    with open(filename, 'r') as file:
        file_contents = file.read()
    file_contents = file_contents.replace(target, replacement)
    with open(filename, 'w') as file:
        file.write(str(file_contents))

def copy_file(source, dest):
    shutil.copyfile(source, dest)

def run(command, cwd = '.', should_log = False):
    if (should_log): logging.debug('Running cmd: %s' % command)
    response = subprocess.run(' '.join(command), cwd=cwd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stderr = response.stderr.decode('utf-8')
    if response.returncode != 0: raise Exception(stderr)
    stdout = response.stdout.decode('utf-8')    
    if (should_log): logging.debug('Response: %s', stdout)    
    return stdout

def generate_models(package_name, spec_file, api_package, tmp_client_folder, out_location, client_fix_hack=lambda: None):
    safe_os_remove(out_location, True)
    # generate the full package
    # See also https://openapi-generator.tech/docs/generators/java
    run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR, 'generate',
         '-g', 'java',
         '-i', spec_file,
         '-o', tmp_client_folder,
         '--additional-properties=packageName={},library=native,hideGenerationTimestamp=true,apiPackage={},invokerPackage={},modelPackage={}'.format(package_name, '{}.generated.api'.format(api_package), COMMON_API_PACKAGE, '{}.generated.models'.format(api_package)),
         '-t', os.path.join(OPENAPI_GENERATION_FOLDER, '5.2.1-template-file-overrides')
         ], should_log=False)

    java_code_root = os.path.join(tmp_client_folder, 'src/main/java/{}/generated'.format(api_package.replace(".", "/")))

    # Copy the bits of code we want
    shutil.copytree(os.path.join(java_code_root, 'models'), os.path.join(out_location, 'models')) 
    safe_os_remove(os.path.join(out_location, 'models', 'AbstractOpenApiSchema.java'))

    logging.info("Successfully generated the %s package" % package_name)

if __name__ == "__main__":    
    logger.info('Will generate models for the Core API')

    # Set working directory to be the same directory as this script
    os.chdir(os.path.dirname(os.path.abspath(__file__)))

    # check & download the openapi-generator.jar
    if not os.path.exists(OPENAPI_GENERATOR_FIXED_VERSION_JAR):
        logger.info('%s does not exist' % OPENAPI_GENERATOR_FIXED_VERSION_JAR)
        logger.info('Will download it from %s...' % OPENAPI_GENERATOR_FIXED_VERSION_DOWNLOAD_URL)        
        urllib.request.urlretrieve(OPENAPI_GENERATOR_FIXED_VERSION_DOWNLOAD_URL, OPENAPI_GENERATOR_FIXED_VERSION_JAR)
        logger.info('Testing the openapi-generator...')
        logger.info(run(['ls %s' % OPENAPI_GENERATION_FOLDER]))
        run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR, 'author'], should_log=False)
        logger.info('All good.')
    
    safe_os_remove(OPENAPI_TEMP_GENERATION_FOLDER, silent=True)
    os.makedirs(OPENAPI_TEMP_GENERATION_FOLDER)

    # download & fix the spec files
    core_api_spec_temp_filename = os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, 'core_api_spec.yaml')
    copy_file(CORE_API_SPEC_LOCATION, core_api_spec_temp_filename)
    replace_in_file(core_api_spec_temp_filename, 'openapi: 3.1.0', 'openapi: 3.0.0')
    logging.info('Loaded Core API Spec from {}'.format(os.path.abspath(CORE_API_SPEC_LOCATION)))

    # generate the clients
    logging.info('Generating code from specs...')

    generate_models("core-api", core_api_spec_temp_filename, CORE_API_PACKAGE, os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, "core-api"), CORE_API_GENERATED_DESTINATION)

    logging.info("Code has been created.")
    
    # clean up  
    safe_os_remove(OPENAPI_TEMP_GENERATION_FOLDER, silent=True)
