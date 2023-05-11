#include "plugin.hpp"

struct Breaker : Module {
    potential::Breaker *inner = NULL;

    enum ParamId { PARAMS_LEN };
    enum InputId { LEFT_INPUT, RIGHT_INPUT, RESET_INPUT, INPUTS_LEN };
    enum OutputId {
        TRIPPED_GATE_OUTPUT,
        LEFT_OUTPUT,
        RIGHT_OUTPUT,
        OUTPUTS_LEN
    };
    enum LightId { TRIPPED_LIGHT, LIGHTS_LEN };

    Breaker() {
        this->inner = potential::breaker_new();
        config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
        configInput(LEFT_INPUT, "Left");
        configInput(RIGHT_INPUT, "Right");
        configInput(RESET_INPUT, "Reset breaker");
        configOutput(TRIPPED_GATE_OUTPUT, "Tripped gate");
        configOutput(LEFT_OUTPUT, "Left");
        configOutput(RIGHT_OUTPUT, "Right");
    }

    ~Breaker() { potential::breaker_free(this->inner); }

    void process(const ProcessArgs &args) override {
        Input *left_in = &inputs[LEFT_INPUT];
        Input *right_in = &inputs[RIGHT_INPUT];
        Input *reset_trigger = &inputs[RESET_INPUT];
        Output *left_out = &outputs[LEFT_OUTPUT];
        Output *right_out = &outputs[RIGHT_OUTPUT];
        Output *tripped_gate = &outputs[TRIPPED_GATE_OUTPUT];

        if (left_in->isConnected() || right_in->isConnected() ||
            reset_trigger->isConnected()) {
            int left_in_len = left_in->getChannels();
            int right_in_len = right_in->getChannels();
            bool tripped_status;

            potential::breaker_process(
                this->inner, left_in->getVoltages(), left_in_len,
                right_in->getVoltages(), right_in_len,
                reset_trigger->getVoltage(), &tripped_status,
                tripped_gate->getVoltages(), left_out->getVoltages(),
                PORT_MAX_CHANNELS, right_out->getVoltages(), PORT_MAX_CHANNELS);
            left_out->setChannels(left_in_len);
            right_out->setChannels(right_in_len);
            lights[TRIPPED_LIGHT].setBrightness(
                static_cast<float>(tripped_status));
        }
    }
};

struct BreakerWidget : ModuleWidget {
    BreakerWidget(Breaker *module) {
        setModule(module);
        setPanel(createPanel(asset::plugin(pluginInstance, "res/Breaker.svg")));

        addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
        addChild(
            createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH,
                                          RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

        addInput(createInputCentered<PJ301MPort>(mm2px(Vec(7.716, 28.665)),
                                                 module, Breaker::LEFT_INPUT));
        addInput(createInputCentered<PJ301MPort>(mm2px(Vec(17.684, 28.665)),
                                                 module, Breaker::RIGHT_INPUT));
        addInput(createInputCentered<PJ301MPort>(mm2px(Vec(12.7, 62.891)),
                                                 module, Breaker::RESET_INPUT));

        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(16.26, 44.0)), module, Breaker::TRIPPED_GATE_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.304, 113.133)), module, Breaker::LEFT_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(17.937, 113.259)), module, Breaker::RIGHT_OUTPUT));

        addChild(createLightCentered<MediumLight<RedLight>>(
            mm2px(Vec(7.006, 44.0)), module, Breaker::TRIPPED_LIGHT));
    }
};

Model *modelBreaker = createModel<Breaker, BreakerWidget>("Breaker");