#include "st3215.h"

using namespace std;

int main() {

  // Initialiser une instance STS3215
  ST3215Handle* handle * st3215_new("/dev/AMC0");

  if (handle == NULL) {
    return -1;
  }

  // Lister les servomoteurs
  vector servo_count = st3215_list_servos(handle);
  if (sizeof(servo_count) / sizeof(servo_count) < 1) {
    cout << "Veuillez brancher un servomoteur !" << endl;
  } else if (servo_count > 1) {
    cout << "Veuillez brancher uniquement un servomoteur !" << endl;
  }

  

  // Fin du programme
  st3215_free(handle);

  return 0;
}
