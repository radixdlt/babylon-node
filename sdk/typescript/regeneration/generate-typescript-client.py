#!/usr/bin/env python3
import urllib.request, logging, subprocess, os, shutil, re, yaml

# Inspired by https://github.com/radixdlt/radixdlt-python-clients
# Requires python 3+ and `pip install pyyaml`

logger = logging.getLogger()
logging.basicConfig(format='%(asctime)s [%(levelname)s]: %(message)s', level=logging.INFO)

API_SCHEMA_LOCATION = '../../../core-rust/core-api-server/core-api-schema.yaml'
API_GENERATED_DESTINATION = '../lib/generated'

OPENAPI_GENERATION_FOLDER='.'
OPENAPI_TEMP_GENERATION_FOLDER='./temp'
OPENAPI_GENERATOR_FIXED_VERSION_JAR=os.path.join(OPENAPI_GENERATION_FOLDER, 'openapi-generator-cli-6.2.1.jar')
OPENAPI_GENERATOR_FIXED_VERSION_DOWNLOAD_URL='https://search.maven.org/remotecontent?filepath=org/openapitools/openapi-generator-cli/6.2.1/openapi-generator-cli-6.2.1.jar'

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

def split_out_inherited_discriminated_types(schema):
    """
    Some context:
    
    The Open API specificaiton is very unclear how to handle discriminated unions, leaving different languages
    and code generators to implement it in different ways.

    The "best" way to do it which allows class-based languages with inheritance to work right is to have allOf on the
    parent class, and have various child classes use anyOd, like this: https://redocly.com/docs/resources/discriminator/#Vehicle.yaml

    BUT many code generators - such as:
    - The TypeScript "open api generator"
    - Ovral - a specialized typescript generator

    Do not support it properly - in particular, to avoid recursive type definitions, you need to essentially add in a "Base" type.
    This method introduces this Base type so that the type generation works correctly.
    """
    types = schema["components"]["schemas"]
    typeRefsChanged = {}
    originalTypeNames = list(types.keys())
    
    # First pass - we need to find all the types that are discriminated unions, and split them
    for typeName in originalTypeNames:
        typeData = types[typeName]
        if "discriminator" in typeData and "type" in typeData and typeData["type"] == "object" and not ("allOf" in typeData or "anyOf" in typeData or "oneOf" in typeData):
            # Assume it's one of the types we're targetting!

            baseTypeData = {
                typeDataKey: typeData[typeDataKey] for typeDataKey in typeData if typeDataKey != "discriminator"
            }

            unionTypeData = {
                # Technically speaking it should be `anyOf` here because the child type doesn't validate the exact key...
                # And so multiple child types could be valid at the same time.
                # But this results in the typescript generator outputting invalid code which doesn't compile (yay).
                # So we stick with "oneOf".
                "oneOf": [
                    { "$ref": childTypeReference } for childTypeReference in typeData["discriminator"]["mapping"].values()
                ],
                "discriminator": typeData["discriminator"],
            }

            baseTypeName = typeName + "Base"
            while baseTypeName in types: # Very basic collision avoidance
                baseTypeName = baseTypeName + "Derived"

            typeRef = "#/components/schemas/" + typeName
            baseTypeRef = "#/components/schemas/" + baseTypeName
            typeRefsChanged[typeRef] = {
                "baseTypeRef": baseTypeRef,
                "discriminator": typeData["discriminator"],
            }

            types[typeName] = unionTypeData
            types[baseTypeName] = baseTypeData

    # Second pass - we go through all the other types, and update their references to the new types
    for typeName in originalTypeNames:
        typeData = types[typeName]
        if "allOf" in typeData and isinstance(typeData["allOf"], list):
            match = None
            for inheritedType in typeData["allOf"]:
                if "$ref" in inheritedType and inheritedType["$ref"] in typeRefsChanged:
                    parentTypeDetails = typeRefsChanged[inheritedType["$ref"]]
                    inheritedType["$ref"] = parentTypeDetails["baseTypeRef"]
                    match = parentTypeDetails
            if match is not None:
                mapping = match["discriminator"]["mapping"]
                ownTag = next((tag for tag in mapping if mapping[tag] == "#/components/schemas/" + typeName), None)
                if ownTag is not None:
                    # Either find the existing inline inner type to amend - or add one
                    lastInnerTypeData = next((
                        innerTypeData for innerTypeData in typeData["allOf"]
                            if "type" in innerTypeData and innerTypeData["type"] == "object"
                            and "properties" in innerTypeData and isinstance(innerTypeData["properties"], dict)
                    ), None)
                    if lastInnerTypeData is None:
                        lastInnerTypeData = { "type": "object", "properties": {} }
                        typeData["allOf"].append(lastInnerTypeData)

                    # Restrict the type of the property name to match the discriminator
                    lastInnerTypeData["properties"][match["discriminator"]["propertyName"]] = {
                        "type": "string",
                        "enum": [ownTag],
                    }


def prepare_schema_for_generation(original_schema_file, api_schema_temp_filename):
    with open(original_schema_file, 'r') as file:
        schema = yaml.safe_load(file)

    # Open API generator only works with 3.0.0
    schema['openapi'] = '3.0.0'
    split_out_inherited_discriminated_types(schema)

    with open(api_schema_temp_filename, 'w') as file:
        yaml.dump(schema, file, sort_keys=False)

def generate_models(prepared_spec_file, tmp_client_folder, out_location):
    safe_os_remove(tmp_client_folder, True)
    # See https://openapi-generator.tech/docs/generators/typescript-fetch/
    run(['java', '-jar', OPENAPI_GENERATOR_FIXED_VERSION_JAR, 'generate',
         '-g', 'typescript-fetch',
         '-i', prepared_spec_file,
         '-o', tmp_client_folder,
         '-t', os.path.join(OPENAPI_GENERATION_FOLDER, 'template-overrides'),
         '--additional-properties=supportsES6=true,modelPropertyNaming=original,npmVersion=0.1.0'
    ], should_log=False)

    logging.info("Successfully generated.")

    def fix_runtime(file_path):
        # For some reason it outputs invalid types for response, this seems to fix it
        replace_in_file(file_path, "let response = undefined;", "let response: Response = undefined as any as Response;")

    fix_runtime(os.path.join(tmp_client_folder, 'runtime.ts'))

    safe_os_remove(out_location, silent=True)
    shutil.copytree(tmp_client_folder, out_location)
    safe_os_remove(tmp_client_folder, silent=True)

    logging.info("Successfully fixed up.")

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

    logging.info('Loading API Schema from {}, and preparing it...'.format(os.path.abspath(API_SCHEMA_LOCATION)))

    api_schema_temp_filename = os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, 'schema.yaml')
    prepare_schema_for_generation(API_SCHEMA_LOCATION, api_schema_temp_filename)

    logging.info('Generating code from prepared schema...')

    generate_models(api_schema_temp_filename, os.path.join(OPENAPI_TEMP_GENERATION_FOLDER, "typescript"), API_GENERATED_DESTINATION)

    logging.info("Code has been created.")

    safe_os_remove(OPENAPI_TEMP_GENERATION_FOLDER, silent=True)

    logging.info("Temp directory cleared.")
