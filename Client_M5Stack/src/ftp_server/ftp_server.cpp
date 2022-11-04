#include "ftp_server_int.h"

FtpServer ftp;  

static void initFtpServer(void);

static void initFtpServer(void)
{
    if (SD.begin()) {
        ftp.begin(FTP_USER, FTP_PASS);
    }
}

void taskFtpServer(void *args)
{
    initFtpServer();

    while (true) {
        ftp.handleFTP();
    }
}
