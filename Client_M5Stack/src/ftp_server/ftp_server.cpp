#include "ftp_server_int.h"

FtpServer ftp;  

static void initFtpServer(void);

static void initFtpServer(void)
{
    // SD は setup() の deviceInitSD() で初期化済み
    ftp.begin(FTP_USER, FTP_PASS);
}

void taskFtpServer(void *args)
{
    initFtpServer();

    while (true) {
        ftp.handleFTP();
        // 他タスク・loop にも CPU を譲る（busy loop 防止）
        delay(1);
    }
}
