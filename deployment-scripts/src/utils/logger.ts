import { Logger } from "tslog";
import { createStream } from "rotating-file-stream";

const stream = createStream("test.log", {
    size: "10M", // rotate every 10 MegaBytes written
    interval: "1d", // rotate daily
    compress: "gzip", // compress rotated files
});

export const logger = new Logger();
logger.attachTransport((logObj) => {
    // Extracting only necessary information from logObj
    const { date, logLevelName, } = logObj._meta;
    const logMessage = { date, logLevelName };

    // Stringify the modified log message
    const formattedLog = logMessage.logLevelName

    // Write the formatted log to the stream
    stream.write(formattedLog + "\n");
});
