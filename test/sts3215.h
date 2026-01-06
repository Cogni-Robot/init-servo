#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define DEFAULT_BAUDRATE 1000000

#define LATENCY_TIMER 50.0

#define TXPACKET_MAX_LEN 250

#define RXPACKET_MAX_LEN 250

#define MIN_POSITION 0

#define MAX_POSITION 4095

#define MAX_SPEED 3400

#define MAX_CORRECTION 2047

#define PKT_HEADER_0 0

#define PKT_HEADER_1 1

#define PKT_ID 2

#define PKT_LENGTH 3

#define PKT_INSTRUCTION 4

#define PKT_ERROR 4

#define PKT_PARAMETER0 5

#define ERRBIT_VOLTAGE 1

#define ERRBIT_ANGLE 2

#define ERRBIT_OVERHEAT 4

#define ERRBIT_OVERELE 8

#define ERRBIT_OVERLOAD 32

#define BROADCAST_ID 254

#define MAX_ID 252

#define STS_END 0

#define INST_PING 1

#define INST_READ 2

#define INST_WRITE 3

#define INST_REG_WRITE 4

#define INST_ACTION 5

#define INST_SYNC_WRITE 131

#define INST_SYNC_READ 130

#define STS_1M 0

#define STS_0_5M 1

#define STS_250K 2

#define STS_128K 3

#define STS_115200 4

#define STS_76800 5

#define STS_57600 6

#define STS_38400 7

#define STS_MODEL_L 3

#define STS_MODEL_H 4

#define STS_ID 5

#define STS_BAUD_RATE 6

#define STS_MIN_ANGLE_LIMIT_L 9

#define STS_MIN_ANGLE_LIMIT_H 10

#define STS_MAX_ANGLE_LIMIT_L 11

#define STS_MAX_ANGLE_LIMIT_H 12

#define STS_CW_DEAD 26

#define STS_CCW_DEAD 27

#define STS_OFS_L 31

#define STS_OFS_H 32

#define STS_MODE 33

#define STS_TORQUE_ENABLE 40

#define STS_ACC 41

#define STS_GOAL_POSITION_L 42

#define STS_GOAL_POSITION_H 43

#define STS_GOAL_TIME_L 44

#define STS_GOAL_TIME_H 45

#define STS_GOAL_SPEED_L 46

#define STS_GOAL_SPEED_H 47

#define STS_LOCK 55

#define STS_PRESENT_POSITION_L 56

#define STS_PRESENT_POSITION_H 57

#define STS_PRESENT_SPEED_L 58

#define STS_PRESENT_SPEED_H 59

#define STS_PRESENT_LOAD_L 60

#define STS_PRESENT_LOAD_H 61

#define STS_PRESENT_VOLTAGE 62

#define STS_PRESENT_TEMPERATURE 63

#define STS_STATUS 65

#define STS_MOVING 66

#define STS_PRESENT_CURRENT_L 69

#define STS_PRESENT_CURRENT_H 70

/**
 * Handle opaque pour ST3215
 */
typedef struct ST3215Handle ST3215Handle;

/**
 * Créer une nouvelle instance ST3215
 *
 * # Arguments
 * * `device` - Chemin du port série (ex: "/dev/ttyUSB0" ou "COM3")
 *
 * # Retour
 * Un pointeur vers ST3215Handle, ou NULL en cas d'erreur
 */
struct ST3215Handle *st3215_new(const char *device);

/**
 * Libérer une instance ST3215
 *
 * # Arguments
 * * `handle` - Handle ST3215 à libérer
 */
void st3215_free(struct ST3215Handle *handle);

/**
 * Vérifier la présence d'un servo
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo (0-253)
 *
 * # Retour
 * 1 si le servo répond, 0 sinon
 */
int32_t st3215_ping_servo(struct ST3215Handle *handle, uint8_t servo_id);

/**
 * Lister tous les servos connectés
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `out_ids` - Buffer pour stocker les IDs trouvés
 * * `max_ids` - Taille maximale du buffer
 *
 * # Retour
 * Nombre de servos trouvés
 */
uintptr_t st3215_list_servos(struct ST3215Handle *handle, uint8_t *out_ids, uintptr_t max_ids);

/**
 * Déplacer un servo vers une position cible
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `position` - Position cible (0-4095)
 * * `speed` - Vitesse de déplacement (0-4095)
 * * `acceleration` - Accélération (0-254)
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_move_to(struct ST3215Handle *handle,
                       uint8_t servo_id,
                       uint16_t position,
                       uint16_t speed,
                       uint8_t acceleration);

/**
 * Lire la position actuelle d'un servo
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_position` - Pointeur pour stocker la position lue
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_position(struct ST3215Handle *handle, uint8_t servo_id, uint16_t *out_position);

/**
 * Lire la vitesse actuelle d'un servo
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_speed` - Pointeur pour stocker la vitesse lue
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_speed(struct ST3215Handle *handle, uint8_t servo_id, uint16_t *out_speed);

/**
 * Lire la charge d'un servo (en pourcentage)
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_load` - Pointeur pour stocker la charge lue
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_load(struct ST3215Handle *handle, uint8_t servo_id, float *out_load);

/**
 * Lire la tension d'un servo (en volts)
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_voltage` - Pointeur pour stocker la tension lue
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_voltage(struct ST3215Handle *handle, uint8_t servo_id, float *out_voltage);

/**
 * Lire le courant d'un servo (en mA)
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_current` - Pointeur pour stocker le courant lu
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_current(struct ST3215Handle *handle, uint8_t servo_id, float *out_current);

/**
 * Lire la température d'un servo (en °C)
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `out_temperature` - Pointeur pour stocker la température lue
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_read_temperature(struct ST3215Handle *handle,
                                uint8_t servo_id,
                                uint8_t *out_temperature);

/**
 * Vérifier si un servo est en mouvement
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 *
 * # Retour
 * 1 si en mouvement, 0 si arrêté, -1 en cas d'erreur
 */
int32_t st3215_is_moving(struct ST3215Handle *handle, uint8_t servo_id);

/**
 * Activer le couple d'un servo
 *
 * # Arguments
 * * `handle` - Handle ST3215
 * * `servo_id` - ID du servo
 * * `enable` - 1 pour activer, 0 pour désactiver
 *
 * # Retour
 * 0 en cas de succès, -1 en cas d'erreur
 */
int32_t st3215_enable_torque(struct ST3215Handle *handle, uint8_t servo_id, int32_t enable);

/**
 * Obtenir la version de la bibliothèque
 *
 * # Retour
 * Chaîne de caractères contenant la version (doit être libérée avec st3215_free_string)
 */
char *st3215_version(void);

/**
 * Libérer une chaîne de caractères allouée par la bibliothèque
 *
 * # Arguments
 * * `s` - Pointeur vers la chaîne à libérer
 */
void st3215_free_string(char *s);
