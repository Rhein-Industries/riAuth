/* Synthetic native regression probe for macOS Seatbelt, never production code. */
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <spawn.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/event.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

static int denied(int result) {
    return result == -1 && (errno == EPERM || errno == EACCES);
}

int main(int argc, char **argv) {
    if (argc > 2 && (!strcmp(argv[1], "inherit") || !strcmp(argv[1], "inherit-socket") || !strcmp(argv[1], "inherit-kqueue"))) {
        int fd = !strcmp(argv[1], "inherit")
            ? open("descriptor-sentinel", O_CREAT | O_RDWR, 0600)
            : !strcmp(argv[1], "inherit-socket")
                ? socket(AF_INET, SOCK_STREAM, 0) : kqueue();
        if (fd < 0 || dup2(fd, 100) != 100) return 2;
        close(fd);
        /* A native inheritable fd, deliberately without FD_CLOEXEC. */
        execv(argv[2], argv + 2);
        return 2;
    }
    if (argc == 2 && !strcmp(argv[1], "fd-closed")) {
        if (fcntl(100, F_GETFD) != -1 || errno != EBADF) return 2;
        puts("descriptor 100 closed");
        return 0;
    }
    if (argc > 1) return 2; /* A wrongly permitted spawn exits promptly. */

    int ok = denied(open("/etc/passwd", O_RDONLY));
    ok &= denied(open("write-sentinel", O_CREAT | O_EXCL | O_WRONLY, 0600));
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock >= 0) {
        struct sockaddr_in addr = {0};
        addr.sin_family = AF_INET;
        addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
        ok &= denied(bind(sock, (struct sockaddr *)&addr, sizeof(addr)));
        close(sock);
    } else {
        ok &= denied(sock);
    }
    pid_t forked = fork();
    if (forked == 0) _exit(2);
    ok &= denied(forked);
    if (forked > 0) waitpid(forked, 0, 0);
    pid_t spawned = 0;
    char *args[] = {argv[0], "child", NULL};
    int spawn_error = posix_spawn(&spawned, argv[0], NULL, NULL, args, environ);
    ok &= spawn_error == EPERM || spawn_error == EACCES;
    if (!spawn_error) waitpid(spawned, 0, 0);
    if (ok) puts("read/write/network/fork/spawn denied");
    return ok ? 0 : 2;
}
