function calculateHash(devEuiString) {
    if (devEuiString.length === 16) {
        const charArray = [...devEuiString];
        let hash = 0;

        charArray.forEach(function (element) {
            hash = hash ^ element;
        })
        return hash;
    }
}

function byteToHex(byte) {
    return ("0" + byte.toString(16).toUpperCase().slice(-2));
}
