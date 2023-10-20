// const downloadProgress = document.getElementById("downloadProgress");
const downloadButton = document.getElementById("downloadButton");
const uploadButton = document.getElementById("uploadButton");

downloadButton.onclick = function() {
    const filename = document.getElementById("filenameTextfield").value;
    // validate filename
    let fileSize;
    try {
        fileSize = getFileSize(filename);
    } catch(error) {
        alert("Error: File not found.");
        return;
    }

    try {
        downloadFile(filename, fileSize);
    } catch(error) {
        alert("Error: Could not download file.");
        return;
    }
}

uploadButton.onclick = function () {
    const file = document.getElementById("uploadFilename").files[0];
    const reader = new FileReader();
    const part_size = 4000000;
    reader.readAsArrayBuffer(file);
    reader.onloadend = (evt) => {
        if (evt.target.readyState === FileReader.DONE) {
            const arrayBuffer = evt.target.result;
            let array = new Uint8Array(arrayBuffer);
            let part = 0;
            let start = 0;
            let end = 0;
            while(true) {
            console.log(part, start, end, array.length);
            start = part * part_size;
            end = part * part_size + part_size;

            if(start >= array.length) {
                return;
            }

            if(end > array.length) {
                end = array.length;
            }
            let bytes = array.slice(start,end);
            let binary = "";
            for (let i = 0; i < bytes.length; i += 1) {
                binary += String.fromCharCode(bytes[i]);
            }
            httpPost("/upload/" + file.name, btoa(binary));
            part+=1;
            }
        }
    }
}

const getFileSize = (filename) => {
    const response = httpGet("/download/" + filename + "?size");
    if(response.status == 200) {
        return response.responseText;
    }
    throw response.responseText;
}

const downloadFile = (filename, fileSize) => {
    let part = 0;
    let byteArrayOffset = 0;
    let byteArray = new Uint8Array(fileSize);
    let base64;
    while(true) {
        base64 = httpGet("/download/" + filename + "?part=" + part++).responseText;
        if(base64.length === 0) {
            saveDataToFile(byteArray, filename);
            return;
        }
        let bytes = base64ToBytes(base64)
        byteArray.set(bytes, byteArrayOffset);
        byteArrayOffset += bytes.length;
        // downloadProgress.value = 100 / file_size * byte_array_offset;
    }
}

const base64ToBytes = (base64) => {
    const binString = atob(base64);
    return Uint8Array.from(binString, (m) => m.codePointAt(0));
}

const saveDataToFile = (data, filename) => {
    const blob = new Blob([data], {type: "application/octet-stream"});
    const blobUrl = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = blobUrl;
    a.download = filename;

    document.body.appendChild(a);
    a.click();

    URL.revokeObjectURL(blobUrl);
    document.body.removeChild(a);
}

const httpGet = (url) => {
    var xmlHttpRequest = new XMLHttpRequest();
    xmlHttpRequest.open("GET", url, false); // false for synchronous request
    xmlHttpRequest.send(null);
    return xmlHttpRequest;
}

const httpPost = (url, data) => {
    var xmlHttpRequest = new XMLHttpRequest();
    xmlHttpRequest.open("POST", url, false); // false for synchronous request
    xmlHttpRequest.send(data);
    return xmlHttpRequest;
}