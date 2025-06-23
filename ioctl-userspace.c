#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>

static int BUFFER_SIZE = 256;

#define RUST_MISC_DEV_FAIL _IO('|', 0)
#define READ_MESSAGE _IOR('|', 0x83, char[BUFFER_SIZE])


int main()
{
  int fd, ret;

  // Open the device file
  printf("Opening /dev/kcounter-rs for reading and writing\n");
  fd = open("/dev/kcounter-rs", O_RDWR);
  if (fd < 0)
  {
    perror("open");
    return errno;
  }

  // Call the unsuccessful ioctl
  printf("Attempting to call in to an non-existent IOCTL\n");
  ret = ioctl(fd, RUST_MISC_DEV_FAIL, NULL);
  if (ret < 0)
  {
    perror("ioctl: Succeeded to fail - this was expected");
  }
  else
  {
    printf("ioctl: Failed to fail\n");
    close(fd);
    return -1;
  }

  char buffer[BUFFER_SIZE];
  ret = ioctl(fd, READ_MESSAGE, &buffer);
  if (ret < 0)
  {
    perror("Failed to read message!");
    close(fd);
    return -1;
  }
  printf("Message received: %s\n", buffer);

  // Close the device file
  printf("Closing /dev/kcounter-rs\n");
  close(fd);

  printf("Success\n");
  return 0;
}