#include "plugin.hpp"

struct MagSign : Module {
    enum ParamId { PARAMS_LEN };
    enum InputId { BIPOLAR_INPUT, MAGNITUDE_INPUT, SIGN_INPUT, INPUTS_LEN };
    enum OutputId {
        MAGNITUDE_OUTPUT,
        SIGN_OUTPUT,
        BIPOLAR_OUTPUT,
        OUTPUTS_LEN
    };
    enum LightId { LIGHTS_LEN };

    MagSign() {
        config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
        configInput(BIPOLAR_INPUT, "");
        configInput(MAGNITUDE_INPUT, "");
        configInput(SIGN_INPUT, "");
        configOutput(MAGNITUDE_OUTPUT, "");
        configOutput(SIGN_OUTPUT, "");
        configOutput(BIPOLAR_OUTPUT, "");
    }

    void process(const ProcessArgs &args) override {
        // Decompose ports (upper half)
        Input *bipolar_in = &inputs[BIPOLAR_INPUT];
        Output *magnitude_out = &outputs[MAGNITUDE_OUTPUT];
        Output *sign_out = &outputs[SIGN_OUTPUT];
        // Recompose ports (lower half)
        Input *magnitude_in = &inputs[MAGNITUDE_INPUT];
        Input *sign_in = &inputs[SIGN_INPUT];
        Output *bipolar_out = &outputs[BIPOLAR_OUTPUT];

        // Upper half (decompose) processing
        if (bipolar_in->isConnected()) {
            // This also determines the polyphony count for outputs.
            int channel_count = bipolar_in->getChannels();

            potential::mag_sign_decompose(
                bipolar_in->getVoltages(), channel_count,
                magnitude_out->getVoltages(), PORT_MAX_CHANNELS,
                sign_out->getVoltages(), PORT_MAX_CHANNELS);

            magnitude_out->setChannels(channel_count);
            sign_out->setChannels(channel_count);
        }

        // Lower half (recompose) processing
        if (magnitude_in->isConnected() || sign_in->isConnected()) {
            int sign_count = sign_in->getChannels();
            int magnitude_count = magnitude_in->getChannels();

            int bipolar_count = potential::mag_sign_recompose(
                magnitude_in->getVoltages(), magnitude_count,
                sign_in->getVoltages(), sign_count, bipolar_out->getVoltages(),
                PORT_MAX_CHANNELS);

            bipolar_out->setChannels(bipolar_count);
        }
    }
};

struct MagSignWidget : ModuleWidget {
    MagSignWidget(MagSign *module) {
        setModule(module);
        setPanel(createPanel(asset::plugin(pluginInstance, "res/MagSign.svg")));

        addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
        addChild(
            createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH,
                                          RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 21.508)), module, MagSign::BIPOLAR_INPUT));
        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 81.372)), module, MagSign::MAGNITUDE_INPUT));
        addInput(createInputCentered<PJ301MPort>(mm2px(Vec(7.62, 97.367)),
                                                 module, MagSign::SIGN_INPUT));

        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 37.504)), module, MagSign::MAGNITUDE_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 53.5)), module, MagSign::SIGN_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 113.363)), module, MagSign::BIPOLAR_OUTPUT));
    }
};

Model *modelMagSign = createModel<MagSign, MagSignWidget>("MagSign");
