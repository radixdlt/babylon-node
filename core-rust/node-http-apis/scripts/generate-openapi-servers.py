#!/usr/bin/env python3
import urllib.request, logging, subprocess, os, shutil, re

# Inspired by https://github.com/radixdlt/radixdlt-python-clients
# Requires python 3+ and various packages above

logger = logging.getLogger()
logging.basicConfig(format='%(asctime)s [%(levelname)s]: %(message)s', level=logging.INFO)

CORE_API_SPEC_LOCATION = '../core-api-schema.yaml'
CORE_API_RUST_GENERATED_DESTINATION = '../src/core_api/generated/'
CORE_API_RUST_PACKAGE = 'core_api::generated'
CORE_API_JAVA_GENERATED_DESTINATION = '../../../core/src/test-core/java/'
CORE_API_JAVA_PACKAGE = 'com.radixdlt.api.core.generated'

OPENAPI_GENERATION_FOLDER='.'
OPENAPI_TEMP_GENERATION_FOLDER='./temp'
OPENAPI_GENERATOR_FIXED_VERSION_JAR=os.path.join(OPENAPI_GENERATION_FOLDER, 'openapi-generator-cli-6.0.1.jar')
OPENAPI_GENERATOR_FIXED_VERSION_DOWNLOAD_URL='https://search.maven.org/remotecontent?filepath=org/openapitools/openapi-generator-cli/6.0.1/openapi-generator-cli-6.0.1.jar'

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

def find_in_file_multiline(filename, regex):
    with open(filename, 'r') as file:
        file_contents = file.read()
    return re.findall(regex, file_contents)

def create_file(filename, file_contents):
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

def generate_rust_models(schema_file, tmp_client_folder, out_location, rust_package):
    safe_os_remove(out_location, True)
    # See https://openapi-generator.tech/docs/generators/rust/
    run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR, 'generate',
         '-g', 'rust',
         '-i', schema_file,
         '-o', tmp_client_folder,
    ], should_log=False)

    logging.info("Successfully generated rust models.")

    rust_code_root = os.path.join(tmp_client_folder, 'src')
    rust_models = os.path.join(rust_code_root, 'models')
    out_models = os.path.join(out_location, 'models')

    os.makedirs(os.path.join(out_location))
    shutil.copytree(rust_models, out_models)

    def get_version_from_oas_file(file_path):
        version_finds = find_in_file_multiline(schema_file, re.compile("  version: '([^']+)'"))
        if len(version_finds) == 0:
            return "UNKNOWN"
        return version_finds[0]

    version = get_version_from_oas_file(schema_file)
    logging.info("Version is: " + version)
    create_file(os.path.join(out_location, 'mod.rs'), "pub mod models;\npub const SCHEMA_VERSION: &str = \"" + version + "\";\n")

    def fix_broken_discriminator_tag(file_path, tag_name):
        # Fix bug that discriminator tags are incorrectly stripped and lower cased
        broken_tag_name = re.sub(r'[^A-Za-z0-9]+', "", tag_name.lower())
        replace_in_file(file_path, 'tag = "' + broken_tag_name +'"', 'tag = "' + tag_name + '"')

    def fix_for_enum_not_implementing_default(file_path, type_name):
        # Fix bug that enums don't implement Default... So replace their Boxes with Options
        regex_pattern = 'pub ([^: ]+): Box<crate::' + rust_package + '::models::' + type_name + '>'
        field_names = find_in_file_multiline(file_path, re.compile(regex_pattern))
        if len(field_names) == 0:
            return
        replace_in_file(file_path, 'Box<crate::' + rust_package + '::models::' + type_name + '>,', 'Option<crate::' + rust_package + '::models::' + type_name + '>, // Using Option permits Default trait; Will always be Some in normal use')
        for field_name in field_names:
            replace_in_file(file_path, field_name + ': Box::new(' + field_name + ')', field_name + ': Option::Some(' + field_name + ')')

    file_names = [file_name for file_name in os.listdir(out_models) if os.path.isfile(os.path.join(out_models, file_name))]
    for file_name in file_names:
        file_path = os.path.join(out_models, file_name)
        # Fix changes due to putting generated files directly into the crate
        replace_in_file(file_path, 'crate::', 'crate::' + rust_package + '::')
        replace_in_file(file_path, ', Serialize, Deserialize', ', serde::Serialize, serde::Deserialize')
        replace_in_file(file_path, '::std::collections::HashMap', '::utils::rust::prelude::IndexMap')
        # Fix bugs in the OAS generation:
        fix_broken_discriminator_tag(file_path, "substate_type")
        fix_broken_discriminator_tag(file_path, "resource_type")
        fix_broken_discriminator_tag(file_path, "key_type")
        fix_broken_discriminator_tag(file_path, "error_type")
        fix_broken_discriminator_tag(file_path, "type")
        fix_broken_discriminator_tag(file_path, "pointer_type")
        fix_broken_discriminator_tag(file_path, "system_type")
        fix_for_enum_not_implementing_default(file_path, "AccessRule")
        fix_for_enum_not_implementing_default(file_path, "AccessRuleNode")
        fix_for_enum_not_implementing_default(file_path, "ProofRule")
        fix_for_enum_not_implementing_default(file_path, "LedgerTransaction")
        fix_for_enum_not_implementing_default(file_path, "ParsedTransaction")
        fix_for_enum_not_implementing_default(file_path, "PublicKey")
        fix_for_enum_not_implementing_default(file_path, "ResourceAmount")
        fix_for_enum_not_implementing_default(file_path, "Signature")
        fix_for_enum_not_implementing_default(file_path, "SignatureWithPublicKey")
        fix_for_enum_not_implementing_default(file_path, "Substate")
        fix_for_enum_not_implementing_default(file_path, "TransactionCallPreviewRequestTarget")
        fix_for_enum_not_implementing_default(file_path, "TransactionReadcallRequestTarget")
        fix_for_enum_not_implementing_default(file_path, "ValidatorTransaction")
        fix_for_enum_not_implementing_default(file_path, "AccessRuleReference")
        fix_for_enum_not_implementing_default(file_path, "LocalMethodReference")
        fix_for_enum_not_implementing_default(file_path, "TransactionSubmitErrorDetails")
        fix_for_enum_not_implementing_default(file_path, "ErrorResponse")
        fix_for_enum_not_implementing_default(file_path, "TargetIdentifier")
        fix_for_enum_not_implementing_default(file_path, "EventEmitterIdentifier")
        fix_for_enum_not_implementing_default(file_path, "TypeInfoDetails")
        fix_for_enum_not_implementing_default(file_path, "StateResourceManager")
        fix_for_enum_not_implementing_default(file_path, "SubstateKey")
        fix_for_enum_not_implementing_default(file_path, "BlueprintCollectionSchemaType")
        fix_for_enum_not_implementing_default(file_path, "BlueprintTypeReferenceKind")
        fix_for_enum_not_implementing_default(file_path, "BlueprintCollectionSchema")
        fix_for_enum_not_implementing_default(file_path, "BlueprintTypeReference")
        fix_for_enum_not_implementing_default(file_path, "Requirement")
        fix_for_enum_not_implementing_default(file_path, "MethodPermission")
        fix_for_enum_not_implementing_default(file_path, "TypePointer")
        fix_for_enum_not_implementing_default(file_path, "OwnerRole")
        fix_for_enum_not_implementing_default(file_path, "FieldSchemaFeatureCondition")
        fix_for_enum_not_implementing_default(file_path, "TransactionMessage")
        fix_for_enum_not_implementing_default(file_path, "PlaintextMessageContent")
        fix_for_enum_not_implementing_default(file_path, "AuthorizedDepositorBadge")
        fix_for_enum_not_implementing_default(file_path, "BlueprintPayloadDef")
        fix_for_enum_not_implementing_default(file_path, "SubstateSystemStructure")
        fix_for_enum_not_implementing_default(file_path, "ObjectSubstateTypeReference")
        fix_for_enum_not_implementing_default(file_path, "GenericSubstitution")
        fix_for_enum_not_implementing_default(file_path, "EntityInfo")
        fix_for_enum_not_implementing_default(file_path, "ResolvedTypeReference")
        fix_for_enum_not_implementing_default(file_path, "BlueprintFieldCondition")
        fix_for_enum_not_implementing_default(file_path, "BlueprintResolvedTypeReference")
        fix_for_enum_not_implementing_default(file_path, "CollectionEntryKey")
        fix_for_enum_not_implementing_default(file_path, "BlueprintFunctionAuthorization")
        fix_for_enum_not_implementing_default(file_path, "BlueprintMethodAuthorization")
        fix_for_enum_not_implementing_default(file_path, "BlueprintRolesDefinition")

    logging.info("Successfully fixed up rust models.")

def generate_java_models(schema_file, tmp_client_folder, out_location, java_package):
    java_package_path = java_package.replace('.', '/')
    safe_os_remove(out_location + java_package_path, True)

    api_package = java_package + '.api'
    invoker_package = java_package + '.client'
    models_package = java_package + '.models'
    # See https://openapi-generator.tech/docs/generators/java
    run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR, 'generate',
         '-g', 'java',
         '-i', schema_file,
         '-o', tmp_client_folder,
         '--additional-properties=openApiNullable=false,useOneOfDiscriminatorLookup=true,library=native,hideGenerationTimestamp=true,apiPackage={},invokerPackage={},modelPackage={}'.format(api_package, invoker_package, models_package),
    ], should_log=False)

    logging.info("Successfully generated java models.")

    code_root = os.path.join(tmp_client_folder, 'src/main/java/' + java_package_path + '/')
    shutil.copytree(code_root, out_location + java_package_path)

    logging.info("Successfully copied java models.")

def fix_spec_and_generate_models(spec_location, rust_destination, rust_package, java_destination, java_package):
    # download & fix the spec files
    spec_temp_location = os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, 'spec.yaml')
    copy_file(spec_location, spec_temp_location)
    replace_in_file(spec_temp_location, 'openapi: 3.1.0', 'openapi: 3.0.0')
    logging.info('Loaded spec from {}'.format(os.path.abspath(spec_location)))

    logging.info('Generating code from spec...')

    generate_rust_models(spec_temp_location, os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, "models-rust"), rust_destination, rust_package)
    generate_java_models(spec_temp_location, os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, "models-java"), java_destination, java_package)

    logging.info("Code has been created.")

    # clean up
    safe_os_remove(OPENAPI_TEMP_GENERATION_FOLDER, silent=True)

if __name__ == "__main__":
    logger.info('Will generate models from the API specifications')

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

    fix_spec_and_generate_models(
        CORE_API_SPEC_LOCATION,
        CORE_API_RUST_GENERATED_DESTINATION,
        CORE_API_RUST_PACKAGE,
        CORE_API_JAVA_GENERATED_DESTINATION,
        CORE_API_JAVA_PACKAGE
    )

