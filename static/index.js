// fix naming
// const downloadProgress = document.getElementById("downloadProgress");
const downloadButton = document.getElementById("downloadButton");
downloadButton.onclick = function() {
    const filename = document.getElementById("filenameTextfield").value;
    try {
        const file_size = get_file_size(filename);
        download_file(filename, file_size); //fix error handling
    } catch(error) {
        console.error("Error while getting file size:" + error);
        alert(error);
    }
}

const get_file_size = (filename) => {
    const response = http_get("/download/" + filename + "?size");
    if(response.status == 200) {
        return response.responseText;
    }
    throw response.responseText;
}

const download_file = (filename, file_size) => {
    let part = 0;
    let byte_array_offset = 0;
    let byte_array = new Uint8Array(file_size);
    let base64;
    while(true) {
        try {
            base64 = http_get("/download/" + filename + "?part=" + part++).responseText;
            if(base64.length === 0) {
                save_data_to_file(byte_array, filename);
                return;
            }
            let bytes = base64_to_bytes(base64)
            byte_array.set(bytes, byte_array_offset);
            byte_array_offset += bytes.length;
            // downloadProgress.value = 100 / file_size * byte_array_offset;
        } catch(error) {
            console.error("Error while processing response:" + error);
        }
    }
}

const base64_to_bytes = (base64) => {
    const binString = atob(base64);
    return Uint8Array.from(binString, (m) => m.codePointAt(0));
}

const save_data_to_file = (data, filename) => {
    const blob = new Blob([data], {type: "application/octet-stream"});
    const blob_url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = blob_url;
    a.download = filename;

    document.body.appendChild(a);
    a.click();

    URL.revokeObjectURL(blob_url);
    document.body.removeChild(a);
}

const http_get = (url) => {
    var xml_http_request = new XMLHttpRequest();
    xml_http_request.open( "GET", url, false ); // false for synchronous request
    xml_http_request.send( null );
    return xml_http_request;
}