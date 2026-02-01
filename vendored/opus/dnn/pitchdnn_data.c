/* Auto generated from checkpoint pitch_vsmallconv1.pth */


#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "pitchdnn.h"
#include "pitchdnn_data.h"


#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#ifndef USE_WEIGHTS_FILE
/* Weight data stripped by strip_weights.py for crate size reduction */
#endif /* USE_WEIGHTS_FILE */

#include <stdio.h>
#ifndef DUMP_BINARY_WEIGHTS
int init_pitchdnn(PitchDNN *model, const WeightArray *arrays) {
    if (linear_init(&model->dense_if_upsampler_1, arrays, "dense_if_upsampler_1_bias", "dense_if_upsampler_1_subias", "dense_if_upsampler_1_weights_int8","dense_if_upsampler_1_weights_float", NULL, NULL, "dense_if_upsampler_1_scale", 88, 64)) return 1;
    if (linear_init(&model->dense_if_upsampler_2, arrays, "dense_if_upsampler_2_bias", "dense_if_upsampler_2_subias", "dense_if_upsampler_2_weights_int8","dense_if_upsampler_2_weights_float", NULL, NULL, "dense_if_upsampler_2_scale", 64, 64)) return 1;
    if (linear_init(&model->dense_downsampler, arrays, "dense_downsampler_bias", "dense_downsampler_subias", "dense_downsampler_weights_int8","dense_downsampler_weights_float", NULL, NULL, "dense_downsampler_scale", 288, 64)) return 1;
    if (linear_init(&model->dense_final_upsampler, arrays, "dense_final_upsampler_bias", "dense_final_upsampler_subias", "dense_final_upsampler_weights_int8","dense_final_upsampler_weights_float", NULL, NULL, "dense_final_upsampler_scale", 64, 192)) return 1;
    if (conv2d_init(&model->conv2d_1, arrays, "conv2d_1_bias", "conv2d_1_weight_float", 1, 4, 3, 3)) return 1;
    if (conv2d_init(&model->conv2d_2, arrays, "conv2d_2_bias", "conv2d_2_weight_float", 4, 1, 3, 3)) return 1;
    if (linear_init(&model->gru_1_input, arrays, "gru_1_input_bias", "gru_1_input_subias", "gru_1_input_weights_int8","gru_1_input_weights_float", NULL, NULL, "gru_1_input_scale", 64, 192)) return 1;
    if (linear_init(&model->gru_1_recurrent, arrays, "gru_1_recurrent_bias", "gru_1_recurrent_subias", "gru_1_recurrent_weights_int8","gru_1_recurrent_weights_float", NULL, NULL, "gru_1_recurrent_scale", 64, 192)) return 1;
    return 0;
}
#endif /* DUMP_BINARY_WEIGHTS */
